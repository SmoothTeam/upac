// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_types::CancelToken;
use upac_types::error::ErrorKind;

use upac_database::meta::MetaStore;

use upac_macro::stage;

use upac_orchestrator::context::Context;
use upac_orchestrator::stage::Stage;

use super::{FileOwner, RequestedPackage, WorkingPrefix};

pub struct ResolveStage;

#[stage]
impl Stage<ErrorKind> for ResolveStage {
    fn run(
        &self, context: &mut Context, _cancel: &CancelToken, _progress: &dyn Fn(Option<&str>, u64, u64),
    ) -> Result<(), ErrorKind> {
        let package = context.take::<RequestedPackage>()?;

        let uuid = context
            .get::<WorkingPrefix>()?
            .database
            .find_package_uuid(&package.0.name, &package.0.arch, package.0.arch_sub.as_deref())?
            .ok_or(ErrorKind::NotFound)?;

        context.put(FileOwner(uuid));

        Ok(())
    }
}
