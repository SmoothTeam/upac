// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_types::CancelToken;
use upac_types::error::ErrorKind;

use upac_deploy::Sysroot;
use upac_deploy::deployment::Deployment;

use upac_macro::stage;

use upac_orchestrator::context::Context;
use upac_orchestrator::stage::Stage;

use super::super::{RequestedPrefixDigestRange, requested_prefix};
use super::DiffPrefixSnapshot;

pub struct PreparingStage;

#[stage]
impl Stage<ErrorKind> for PreparingStage {
    fn run(
        &self, context: &mut Context, _cancel: &CancelToken, _progress: &dyn Fn(Option<&str>, u64, u64),
    ) -> Result<(), ErrorKind> {
        let sysroot = context.get::<Sysroot>()?;
        let requested = context.get::<RequestedPrefixDigestRange>()?;

        let from_digest = requested_prefix(sysroot, requested.from.as_ref())?.digest().clone();
        let to_digest = requested_prefix(sysroot, requested.to.as_ref())?.digest().clone();

        let changed = sysroot
            .repo()
            .open_tree(&from_digest)?
            .diff(&sysroot.repo().open_tree(&to_digest)?);
        let from_database = sysroot.prefix_database(&from_digest)?;
        let to_database = sysroot.prefix_database(&to_digest)?;

        context.put(DiffPrefixSnapshot {
            changed,
            from_database,
            to_database,
        });

        Ok(())
    }
}
