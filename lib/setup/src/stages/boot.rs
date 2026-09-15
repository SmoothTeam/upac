// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::fs::{File, copy};
use std::io::Read;

use composefs::erofs::reader::erofs_to_filesystem;
use composefs::fsverity::FsVerityHashValue;

use upac::boot::{WrittenBootEntry, write_boot_entry};
use upac::layout::boot::{UPAC_UKI_FROM_SLOT, UPAC_UKI_TO_SLOT};
use upac::orchestrator::context::{Context, ctx_get};
use upac::orchestrator::stage::{NoRollback, RollbackGuard, Stage, StageResult};
use upac::plugin::boot::BootPlugins;

use upac_abi::hook::CancelToken;

use upac_types::hook::ProgressEventBuilder;
use upac_types::request::{BootPluginInstallRequest, BootPluginSetOneShotRequest};

use super::{DeployDigests, RequestedBootPlugin};

use crate::error::SetupError;
use crate::layout::genesis::EFI_LINUX_DIR;
use crate::target::TargetSysroot;

pub struct StageBootStage;

impl Stage<SetupError> for StageBootStage {
    fn run(
        &self, context: &mut Context, _cancel: &CancelToken, progress: ProgressEventBuilder,
    ) -> Result<(ProgressEventBuilder, StageResult, Box<dyn RollbackGuard>), SetupError> {
        let target = ctx_get!(context, TargetSysroot);
        let requested_boot_plugin = ctx_get!(context, RequestedBootPlugin);
        let deploy_digests = ctx_get!(context, DeployDigests);

        let repository = target.repository();
        let prefix_digest_hex = deploy_digests.prefix.to_hex();
        let esp_mount_point = target.esp_mount_point();

        let plugin = BootPlugins::new()?.load(requested_boot_plugin)?;

        plugin.install(BootPluginInstallRequest {
            esp_mount_point: esp_mount_point.to_string_lossy().into_owned(),
            esp_partition_number: target.esp_partition_number(),
            esp_starting_lba: target.esp_starting_lba(),
            esp_ending_lba: target.esp_ending_lba(),
            esp_unique_partition_guid: target.esp_unique_partition_guid().to_bytes_le(),
            to_slot: UPAC_UKI_TO_SLOT.to_owned(),
            from_slot: UPAC_UKI_FROM_SLOT.to_owned(),
        })?;

        let (image, _enable_verity) = repository.open_image(prefix_digest_hex.as_str())?;

        let mut data = Vec::new();
        File::from(image).read_to_end(&mut data)?;

        let prefix_tree = erofs_to_filesystem(&data)?;

        let written = write_boot_entry(
            repository,
            &prefix_tree,
            deploy_digests.prefix.clone(),
            &esp_mount_point,
            &prefix_digest_hex,
        )?;

        if matches!(written, WrittenBootEntry::Uki(_)) {
            let efi_linux = esp_mount_point.join(EFI_LINUX_DIR);
            let to_path = efi_linux.join(format!("{UPAC_UKI_TO_SLOT}.efi"));
            let from_path = efi_linux.join(format!("{UPAC_UKI_FROM_SLOT}.efi"));
            copy(&to_path, &from_path)?;
        }

        plugin.set_one_shot(BootPluginSetOneShotRequest {
            entry_name: written.into_entry_name(),
        })?;

        Ok((progress, StageResult::Advance, Box::new(NoRollback)))
    }
}
