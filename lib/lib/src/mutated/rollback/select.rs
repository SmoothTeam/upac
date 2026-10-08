// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_types::CancelToken;
use upac_types::error::ErrorKind;

use upac_composefs::fs::WrittenFile;

use upac_deploy::Sysroot;
use upac_deploy::deployment::{Deployment, PrefixDeploy};
use upac_deploy::etc::SetAsideEtc;

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

        let set_aside = if sysroot.is_live_etc_modified(prefix.digest())? {
            let is_running = sysroot.running_prefix()?.digest() == prefix.digest();
            if !requested.discard_etc_changes || is_running {
                return Err(ErrorKind::AlreadyExists);
            }

            Some(sysroot.set_aside_live_etc(prefix.digest())?)
        } else {
            None
        };

        let writes = match Self::switch_config(sysroot, &mut prefix, config_index) {
            Ok(writes) => writes,
            Err(error) => {
                if let Some(set_aside) = &set_aside {
                    set_aside.restore()?;
                }
                return Err(error);
            }
        };

        context.put(SelectionWrites(writes));
        context.put(BootTarget(prefix.digest().clone()));
        if let Some(set_aside) = set_aside {
            context.put::<SetAsideEtc>(set_aside);
        }

        Ok(())
    }

    fn rollback(&self, context: &mut Context) -> Result<(), ErrorKind> {
        if let Ok(writes) = context.take::<SelectionWrites>() {
            for written in writes.0.iter().rev() {
                written.restore()?;
            }
        }

        if let Ok(set_aside) = context.take::<SetAsideEtc>() {
            set_aside.restore()?;
        }

        Ok(())
    }
}

impl SelectStage {
    fn switch_config(
        sysroot: &Sysroot, prefix: &mut PrefixDeploy, config_index: usize,
    ) -> Result<Vec<WrittenFile>, ErrorKind> {
        prefix.switch_config(config_index)?;

        let meta_written = sysroot.save_prefix(prefix)?;
        match sysroot.set_next_prefix(prefix) {
            Ok(next_written) => Ok(vec![meta_written, next_written]),
            Err(error) => {
                meta_written.restore()?;
                Err(error.into())
            }
        }
    }
}
