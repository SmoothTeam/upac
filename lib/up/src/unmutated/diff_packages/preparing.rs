// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_types::CancelToken;
use upac_types::error::ErrorKind;

use upac_database::meta::MetaStore;

use upac_deploy::Sysroot;
use upac_deploy::deployment::Deployment;

use upac_macro::stage;

use upac_orchestrator::context::Context;
use upac_orchestrator::stage::Stage;

use super::super::{RequestedPrefixDigestRange, requested_prefix};
use super::DiffPackagesSnapshot;

pub struct PreparingStage;

#[stage]
impl Stage<ErrorKind> for PreparingStage {
    fn run(
        &self, context: &mut Context, _cancel: &CancelToken, _progress: &dyn Fn(Option<&str>, u64, u64),
    ) -> Result<(), ErrorKind> {
        let sysroot = context.get::<Sysroot>()?;
        let requested = context.get::<RequestedPrefixDigestRange>()?;

        let from_prefix = requested_prefix(sysroot, requested.from.as_ref())?;
        let to_prefix = requested_prefix(sysroot, requested.to.as_ref())?;

        let from = sysroot.prefix_database(from_prefix.digest())?.list_packages_metas()?;
        let to = sysroot.prefix_database(to_prefix.digest())?.list_packages_metas()?;

        context.put(DiffPackagesSnapshot { from, to });

        Ok(())
    }
}
