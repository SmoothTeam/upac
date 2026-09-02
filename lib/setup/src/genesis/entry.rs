// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::fs::File;
use std::io::Read;

use composefs::erofs::reader::erofs_to_filesystem;
use composefs::fsverity::FsVerityHashValue;
use composefs::repository::Repository;
use composefs::tree::FileSystem;

use upac::boot::write_boot_entry;
use upac::composefs::repository::ObjectID;
use upac::errors::CommonError;
use upac::layout::boot_plugins::{BOOT_PLUGINS_DIR, MANIFEST_EXTENSION};
use upac::orchestrator::Context;
use upac::orchestrator::stage::{NoRollback, RollbackGuard, Stage, StageResult};
use upac::plugin::boot::resolve_boot_plugin;

use upac_abi::hook::{CancelToken, ProgressEventBuilder};

use crate::error::SetupError;
use crate::target::TargetSysroot;
use crate::types::{GenesisInput, PrefixDigest};

pub struct StageBootStage;

impl Stage<SetupError> for StageBootStage {
    fn run(
        &self, context: &mut Context, _cancel: &CancelToken, progress: ProgressEventBuilder,
    ) -> Result<(ProgressEventBuilder, StageResult, Box<dyn RollbackGuard>), SetupError> {
        let target = context.get::<TargetSysroot>().ok_or(CommonError::MissingResult)?;
        let input = context.get::<GenesisInput>().ok_or(CommonError::MissingResult)?;
        let prefix_digest = context.get::<PrefixDigest>().ok_or(CommonError::MissingResult)?;

        let repository = target.repository();
        let prefix_digest_hex = prefix_digest.0.to_hex();

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
