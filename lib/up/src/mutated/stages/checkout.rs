// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_types::CancelToken;
use upac_types::error::ErrorKind;

use upac_boot_loader::BootPlugins;

use upac_deploy::Sysroot;
use upac_deploy::boot::find_esp_mount;

use upac_macro::stage;

use upac_orchestrator::context::Context;
use upac_orchestrator::stage::Stage;

use super::{BootTarget, RequestedBootPlugin, ResolvedBootEntry};

pub struct CheckoutStage;

#[stage]
impl Stage<ErrorKind> for CheckoutStage {
    fn run(
        &self, context: &mut Context, _cancel: &CancelToken, _progress: &dyn Fn(Option<&str>, u64, u64),
    ) -> Result<(), ErrorKind> {
        let sysroot = context.get::<Sysroot>()?;
        let target = context.get::<BootTarget>()?;

        let plugin = BootPlugins::new()?.load(&context.get::<RequestedBootPlugin>()?.0)?;
        let written = sysroot.write_boot_entry(&target.0, &find_esp_mount()?, plugin.boot_resource_kind()?)?;

        context.put(ResolvedBootEntry {
            plugin,
            entry_name: written.into_entry_name(),
        });

        Ok(())
    }
}
