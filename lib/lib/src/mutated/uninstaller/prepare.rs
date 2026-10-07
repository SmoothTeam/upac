// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_types::CancelToken;
use upac_types::decoder::PackageTriggers;
use upac_types::error::ErrorKind;

use upac_database::meta::MetaStore;
use upac_database::triggers::TriggerStore;

use upac_deploy::working::WorkingPrefix;

use upac_macro::stage;

use upac_orchestrator::context::Context;
use upac_orchestrator::stage::Stage;

use super::{RemovalTarget, RequestedPackages};

pub struct PrepareStage;

#[stage]
impl Stage<ErrorKind> for PrepareStage {
    fn run(
        &self, context: &mut Context, _cancel: &CancelToken, _progress: &dyn Fn(Option<&str>, u64, u64),
    ) -> Result<(), ErrorKind> {
        let requested = context.take::<RequestedPackages>()?;
        let database = context.get::<WorkingPrefix>()?.database();

        let mut targets = Vec::new();
        let mut triggers = Vec::new();
        for package in &requested.0 {
            let uuid = database
                .find_package_uuid(&package.name, &package.arch, package.arch_sub.as_deref())?
                .ok_or(ErrorKind::NotFound)?;

            if let Some(package_triggers) = database.get_package_triggers(uuid)? {
                triggers.push(package_triggers);
            }
            targets.push(RemovalTarget(uuid));
        }

        context.put::<Vec<RemovalTarget>>(targets);
        context.put::<Vec<PackageTriggers>>(triggers);

        Ok(())
    }
}
