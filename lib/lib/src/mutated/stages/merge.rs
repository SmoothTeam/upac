// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_types::CancelToken;
use upac_types::error::ErrorKind;

use upac_composefs::tree::Tree;

use upac_deploy::Sysroot;
use upac_deploy::deployment::Deployment;
use upac_deploy::deployment::config::ConfigDeploy;

use upac_macro::stage;

use upac_orchestrator::context::Context;
use upac_orchestrator::stage::Stage;

use super::{CommitInfo, NewConfig, NewDefaults, RunningPrefix};

use crate::layout::prefix::DEFAULTS_DIR;

pub struct MergeStage;

#[stage]
impl Stage<ErrorKind> for MergeStage {
    fn run(
        &self, context: &mut Context, _cancel: &CancelToken, progress: &dyn Fn(Option<&str>, u64, u64),
    ) -> Result<(), ErrorKind> {
        let new_defaults = context.take::<NewDefaults>()?;
        let sysroot = context.get::<Sysroot>()?;
        let running_prefix = &context.get::<RunningPrefix>()?.0;
        let commit_info = context.get::<CommitInfo>()?;

        let running_tree = sysroot.repo().open_tree(running_prefix.digest())?;
        let base_defaults = if running_tree.contains(DEFAULTS_DIR) {
            running_tree.copy_tree(DEFAULTS_DIR)?
        } else {
            sysroot.repo().empty_tree()
        };

        let current_config = running_prefix.current_config().ok_or(ErrorKind::NotFound)?;
        let mut live_config = sysroot.repo().open_tree(current_config.digest())?;
        live_config.apply_overlay_upper(&sysroot.live_etc_upper_dir(running_prefix.digest()))?;

        let merged = Tree::merge(
            &base_defaults,
            &new_defaults.0,
            &live_config,
            commit_info.allow_conflict_files,
        )?;

        let conflicts_total = merged.conflicts.len() as u64;
        for (position, path) in merged.conflicts.iter().enumerate() {
            progress(
                Some(path.to_string_lossy().as_ref()),
                position as u64 + 1,
                conflicts_total,
            );
        }

        let config = ConfigDeploy::new(
            merged.tree.commit()?,
            commit_info.subject.clone(),
            commit_info.message.clone(),
        );
        context.put(NewConfig(config));

        Ok(())
    }
}
