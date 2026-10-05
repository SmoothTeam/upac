// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_types::CancelToken;
use upac_types::error::ErrorKind;

use upac_deploy::Sysroot;
use upac_deploy::deployment::Deployment;
use upac_deploy::deployment::config::ConfigDeploy;

use upac_macro::stage;

use upac_orchestrator::context::Context;
use upac_orchestrator::stage::Stage;

use super::CommitInfo;

pub struct TransactionStage;

#[stage]
impl Stage<ErrorKind> for TransactionStage {
    fn run(
        &self, context: &mut Context, _cancel: &CancelToken, _progress: &dyn Fn(Option<&str>, u64, u64),
    ) -> Result<(), ErrorKind> {
        let sysroot = context.get::<Sysroot>()?;
        let commit_info = context.get::<CommitInfo>()?;

        let mut running_prefix = sysroot.running_prefix()?;
        let current_config_digest = running_prefix
            .current_config()
            .ok_or(ErrorKind::NotFound)?
            .digest()
            .clone();

        let mut live_config = sysroot.repo().open_tree(&current_config_digest)?;
        live_config.apply_overlay_upper(&sysroot.live_etc_upper_dir(running_prefix.digest()))?;
        let live_config_digest = live_config.commit()?;

        if live_config_digest == current_config_digest {
            return Err(ErrorKind::AlreadyExists);
        }

        running_prefix.add_config(ConfigDeploy::new(
            live_config_digest,
            commit_info.subject.clone(),
            commit_info.message.clone(),
        ));
        sysroot.save_prefix(&running_prefix)?;

        Ok(())
    }
}
