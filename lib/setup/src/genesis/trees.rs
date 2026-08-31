// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::env::temp_dir;
use std::fs::{File, write};
use std::path::PathBuf;

use composefs::generic_tree::Stat;
use composefs::repository::ImportContext;
use composefs::tree::FileSystem;

use upac::composefs::file::FileHandle;
use upac::composefs::repository::commit_tree;
use upac::database::files::FileStoreMut;
use upac::database::meta::MetaStoreMut;
use upac::database::{InMemory, MemoryDatabase};
use upac::errors::CommonError;
use upac::layout::database::DATABASE_PATH;
use upac::orchestrator::Context;
use upac::orchestrator::stage::{NoRollback, RollbackGuard, Stage};

use upac_abi::hook::{CancelToken, ProgressEventBuilder};

use upac_types::{FileEntry, FileEntryScope, PackageMeta};

use crate::error::SetupError;
use crate::genesis::{ConfigDigest, GenesisInput, PrefixDigest, ResolvedSourceDir};
use crate::layout::genesis::SCRATCH_FILENAME;
use crate::target::TargetSysroot;

pub struct ImportTreesStage;

impl Stage<SetupError> for ImportTreesStage {
    fn run(
        &self, context: &mut Context, cancel: &CancelToken, progress: ProgressEventBuilder,
    ) -> Result<(ProgressEventBuilder, Box<dyn RollbackGuard>), SetupError> {
        let meta = context.take::<PackageMeta>().ok_or(CommonError::MissingResult)?;
        let target = context.get::<TargetSysroot>().ok_or(CommonError::MissingResult)?;
        let input = context.get::<GenesisInput>().ok_or(CommonError::MissingResult)?;
        let resolved = context.get::<ResolvedSourceDir>().ok_or(CommonError::MissingResult)?;

        let repository = target.repository();
        let mut import_ctx = ImportContext::default();

        let mut prefix_tree = FileSystem::new(Stat::uninitialized());
        let usr_source = resolved.0.join("usr");
        let imported = if usr_source.is_dir() {
            FileHandle::new(PathBuf::new()).import_directory(
                repository,
                &mut prefix_tree,
                &usr_source,
                &mut import_ctx,
                cancel,
            )?
        } else {
            Vec::new()
        };

        let mut config_tree = FileSystem::new(Stat::uninitialized());
        let config_source = resolved.0.join("etc");
        let imported_config = if !input.empty_config && config_source.is_dir() {
            FileHandle::new(PathBuf::new()).import_directory(
                repository,
                &mut config_tree,
                &config_source,
                &mut import_ctx,
                cancel,
            )?
        } else {
            Vec::new()
        };

        let mut database = MemoryDatabase::new_in_memory()?;
        let uuid = database.insert_package_meta(&meta)?;

        for path in imported {
            database.insert_package_file(
                uuid,
                &FileEntry {
                    path: path.to_string_lossy().into_owned(),
                    is_user: false,
                    scope: FileEntryScope::Prefix,
                },
            )?;
        }
        for path in imported_config {
            database.insert_package_file(
                uuid,
                &FileEntry {
                    path: path.to_string_lossy().into_owned(),
                    is_user: false,
                    scope: FileEntryScope::Config,
                },
            )?;
        }

        let database_bytes = database.into_bytes()?;
        let database_scratch_path = temp_dir().join(SCRATCH_FILENAME);
        write(&database_scratch_path, &database_bytes)?;

        FileHandle::new(DATABASE_PATH).insert_file(
            repository,
            &mut prefix_tree,
            &File::open(&database_scratch_path)?,
            Stat::uninitialized(),
            &mut import_ctx,
        )?;

        let prefix_digest = commit_tree(repository, prefix_tree)?;
        let config_digest = commit_tree(repository, config_tree)?;

        context.put(PrefixDigest(prefix_digest));
        context.put(ConfigDigest(config_digest));

        Ok((progress, Box::new(NoRollback)))
    }
}
