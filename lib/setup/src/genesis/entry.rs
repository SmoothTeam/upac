// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::fs::{File, copy, create_dir_all};
use std::io::Read;

use composefs::erofs::reader::erofs_to_filesystem;
use composefs::fsverity::FsVerityHashValue;
use composefs::repository::Repository;
use composefs::tree::FileSystem;

use upac::boot::write_boot_entry;
use upac::composefs::repository::ObjectID;
use upac::layout::boot_plugins::{BOOT_PLUGINS_DIR, MANIFEST_EXTENSION};
use upac::orchestrator::Context;
use upac::orchestrator::stage::{NoRollback, RollbackGuard, Stage, StageResult};
use upac::plugin::boot::resolve_boot_plugin;

use upac_abi::hook::{CancelToken, ProgressEventBuilder};

use super::ctx_get;

use crate::error::SetupError;
use crate::layout::genesis::{ESP_FALLBACK_LOADER, REFIND_SOURCE, SYSTEMD_BOOT_SOURCE};
use crate::target::TargetSysroot;
use crate::types::{GenesisInput, PrefixDigest, ResolvedSourceDir};

pub struct StageBootStage;

impl Stage<SetupError> for StageBootStage {
    fn run(
        &self, context: &mut Context, _cancel: &CancelToken, progress: ProgressEventBuilder,
    ) -> Result<(ProgressEventBuilder, StageResult, Box<dyn RollbackGuard>), SetupError> {
        let target = ctx_get!(context, TargetSysroot);
        let input = ctx_get!(context, GenesisInput);
        let prefix_digest = ctx_get!(context, PrefixDigest);
        let resolved = ctx_get!(context, ResolvedSourceDir);

        let repository = target.repository();
        let prefix_digest_hex = prefix_digest.0.to_hex();

        let candidate = match input.boot_plugin.as_deref() {
            Some("systemd-boot") => Some(SYSTEMD_BOOT_SOURCE),
            Some("refind") => Some(REFIND_SOURCE),
            _ => None,
        };

        if let Some(candidate) = candidate {
            let source = resolved.0.join(candidate);
            let destination = target.esp_mount_point().join(ESP_FALLBACK_LOADER);
            if let Some(parent) = destination.parent() {
                create_dir_all(parent)?;
            }
            copy(&source, &destination)?;
        }

        let prefix_tree = Self::reopen_tree(repository, &prefix_digest_hex)?;
        let entry_name = write_boot_entry(
            repository,
            &prefix_tree,
            prefix_digest.0.clone(),
            &target.esp_mount_point(),
            &prefix_digest_hex,
        )?;

        let plugin = resolve_boot_plugin(BOOT_PLUGINS_DIR, MANIFEST_EXTENSION, input.boot_plugin.as_deref())?;
        plugin.set_one_shot(&entry_name)?;

        Ok((progress, StageResult::Advance, Box::new(NoRollback)))
    }
}

impl StageBootStage {
    fn reopen_tree(repository: &Repository<ObjectID>, digest: &str) -> Result<FileSystem<ObjectID>, SetupError> {
        let (image, _enable_verity) = repository.open_image(digest)?;

        let mut data = Vec::new();
        File::from(image).read_to_end(&mut data)?;

        Ok(erofs_to_filesystem(&data)?)
    }
}
