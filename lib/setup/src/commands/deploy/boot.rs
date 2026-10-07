// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_types::CancelToken;
use upac_types::error::ErrorKind;
use upac_types::request::booter::{BootPluginInstallRequest, BootPluginSetOneShotRequest};

use upac_boot_loader::BootPlugins;

use upac_deploy::Sysroot;
use upac_deploy::layout::boot::{UPAC_UKI_FROM_SLOT, UPAC_UKI_TO_SLOT};

use upac_macro::stage;

use upac_orchestrator::context::Context;
use upac_orchestrator::stage::Stage;

use super::{EspMountPoint, RequestedBootPlugin, RequestedDeploy};

use crate::gpt::Partition;

pub struct BootStage;

#[stage]
impl Stage<ErrorKind> for BootStage {
    fn run(
        &self, context: &mut Context, _cancel: &CancelToken, _progress: &dyn Fn(Option<&str>, u64, u64),
    ) -> Result<(), ErrorKind> {
        let esp = context.get::<Partition>()?;
        let esp_mount_point = &context.get::<EspMountPoint>()?.0;

        let plugin = BootPlugins::new()?.load(&context.get::<RequestedBootPlugin>()?.0)?;
        let written = context.get::<Sysroot>()?.write_boot_entry(
            &context.get::<RequestedDeploy>()?.prefix_digest,
            esp_mount_point,
            plugin.boot_resource_kind()?,
        )?;

        plugin.install(BootPluginInstallRequest {
            esp_mount_point: esp_mount_point.to_string_lossy().into_owned(),
            esp_partition_number: esp.number(),
            esp_starting_lba: esp.first_lba(),
            esp_ending_lba: esp.last_lba(),
            esp_unique_partition_guid: esp.unique_guid().to_bytes_le(),
            to_slot: UPAC_UKI_TO_SLOT.to_owned(),
            from_slot: UPAC_UKI_FROM_SLOT.to_owned(),
        })?;

        plugin.set_one_shot(BootPluginSetOneShotRequest {
            entry_name: written.into_entry_name(),
        })?;

        Ok(())
    }
}
