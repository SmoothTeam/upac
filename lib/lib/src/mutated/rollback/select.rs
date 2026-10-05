// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::fs::{create_dir_all, read_dir, remove_dir_all};
use std::io::ErrorKind as IoErrorKind;

use upac_types::CancelToken;
use upac_types::error::ErrorKind;

use upac_deploy::Sysroot;
use upac_deploy::deployment::Deployment;

use upac_macro::stage;

use upac_orchestrator::context::Context;
use upac_orchestrator::stage::Stage;

use super::super::stages::BootTarget;
use super::{RequestedConfig, SelectionWrites};

pub struct SelectStage;

#[stage]
impl Stage<ErrorKind> for SelectStage {
    fn run(
        &self, context: &mut Context, _cancel: &CancelToken, _progress: &dyn Fn(Option<&str>, u64, u64),
    ) -> Result<(), ErrorKind> {
        let sysroot = context.get::<Sysroot>()?;
        let requested = context.get::<RequestedConfig>()?;

        let (mut prefix, config_index) = sysroot
            .prefixes()?
            .into_iter()
            .find_map(|prefix| {
                let config_index = prefix
                    .configs()
                    .iter()
                    .position(|config| config.digest() == &requested.config_digest)?;
                Some((prefix, config_index))
            })
            .ok_or(ErrorKind::NotFound)?;

        let upper_dir = sysroot.live_etc_upper_dir(prefix.digest());
        let has_changes = match read_dir(&upper_dir) {
            Ok(mut entries) => entries.next().is_some(),
            Err(error) if error.kind() == IoErrorKind::NotFound => false,
            Err(error) => return Err(error.into()),
        };

        if has_changes {
            let is_running = sysroot.running_prefix()?.digest() == prefix.digest();
            if !requested.discard_etc_changes || is_running {
                return Err(ErrorKind::AlreadyExists);
            }

            remove_dir_all(&upper_dir)?;
            create_dir_all(&upper_dir)?;
        }

        prefix.switch_config(config_index)?;

        let meta_written = sysroot.save_prefix(&prefix)?;
        let next_written = match sysroot.set_next_prefix(&prefix) {
            Ok(next_written) => next_written,
            Err(error) => {
                meta_written.restore()?;
                return Err(error.into());
            }
        };

        let target = prefix.digest().clone();
        context.put(SelectionWrites(vec![meta_written, next_written]));
        context.put(BootTarget(target));

        Ok(())
    }

    fn rollback(&self, context: &mut Context) -> Result<(), ErrorKind> {
        let Ok(writes) = context.take::<SelectionWrites>() else {
            return Ok(());
        };

        for written in writes.0.iter().rev() {
            written.restore()?;
        }

        Ok(())
    }
}
