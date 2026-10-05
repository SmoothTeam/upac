// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_types::CancelToken;
use upac_types::error::ErrorKind;

use upac_composefs::Digest;

use upac_deploy::Sysroot;
use upac_deploy::deployment::Deployment;

use upac_macro::stage;

use upac_orchestrator::context::Context;
use upac_orchestrator::stage::Stage;

use super::super::{RequestedConfigDigestRange, requested_prefix_config};
use super::DiffConfigSnapshot;

pub struct PreparingStage;

#[stage]
impl Stage<ErrorKind> for PreparingStage {
    fn run(
        &self, context: &mut Context, _cancel: &CancelToken, _progress: &dyn Fn(Option<&str>, u64, u64),
    ) -> Result<(), ErrorKind> {
        let sysroot = context.get::<Sysroot>()?;
        let requested = context.get::<RequestedConfigDigestRange>()?;

        let (from_config_digest, from_prefix_digest) = Self::locate_config(sysroot, requested.from.as_ref())?;
        let (to_config_digest, to_prefix_digest) = Self::locate_config(sysroot, requested.to.as_ref())?;

        let changed = sysroot
            .repo()
            .open_tree(&from_config_digest)?
            .diff(&sysroot.repo().open_tree(&to_config_digest)?);
        let from_database = sysroot.prefix_database(&from_prefix_digest)?;
        let to_database = sysroot.prefix_database(&to_prefix_digest)?;

        context.put(DiffConfigSnapshot {
            changed,
            from_database,
            to_database,
        });

        Ok(())
    }
}

impl PreparingStage {
    fn locate_config(sysroot: &Sysroot, config_digest: Option<&Digest>) -> Result<(Digest, Digest), ErrorKind> {
        let Some(config_digest) = config_digest else {
            let running_prefix = sysroot.running_prefix()?;
            let current_config = requested_prefix_config(&running_prefix, None)?;

            return Ok((current_config.digest().clone(), running_prefix.digest().clone()));
        };

        let owner_prefix = sysroot
            .prefixes()?
            .into_iter()
            .find(|prefix| prefix.configs().iter().any(|config| config.digest() == config_digest))
            .ok_or(ErrorKind::NotFound)?;

        Ok((config_digest.clone(), owner_prefix.digest().clone()))
    }
}
