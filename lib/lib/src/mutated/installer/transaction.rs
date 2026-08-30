// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::fs::{File, write};
use std::path::{Path, PathBuf};

use composefs::fsverity::FsVerityHashValue;
use composefs::generic_tree::Stat;
use composefs::repository::{ImportContext, Repository};
use composefs::tree::FileSystem;

use upac_abi::hook::{CancelToken, ProgressEventBuilder};

use upac_types::{DeclarativeTrigger, FileEntry, FileEntryScope, PackageTemp, TmpPath};

use crate::composefs::error::RepoError;
use crate::composefs::file::FileHandle;
use crate::composefs::repository::{ObjectID, commit_tree};
use crate::database::files::FileStoreMut;
use crate::database::meta::MetaStoreMut;
use crate::database::triggers::TriggerStoreMut;
use crate::database::{InMemory, MemoryDatabase};
use crate::deploy::Deploy;
use crate::deploy::digest::current_prefix_digest;
use crate::errors::CommonError;
use crate::layout::database::{DATABASE_PATH, INSTALLER_SCRATCH_FILENAME};
use crate::mutated::installer::{InstallError, NewConfigDefaults, NewPrefixDigest};
use crate::orchestrator::Context;
use crate::orchestrator::stage::{NoRollback, RollbackGuard, Stage};

pub struct TransactionStage;

struct ImportPackageData<'a> {
    repository: &'a Repository<ObjectID>,
    tree: &'a mut FileSystem<ObjectID>,
    config_defaults: &'a mut FileSystem<ObjectID>,
    database: &'a mut MemoryDatabase,
    import_ctx: &'a mut ImportContext,
    cancel: &'a CancelToken,
}

impl Stage<InstallError> for TransactionStage {
    fn run(
        &self, context: &mut Context, cancel: &CancelToken, mut progress: ProgressEventBuilder,
    ) -> Result<(ProgressEventBuilder, Box<dyn RollbackGuard>), InstallError> {
        let packages = context.take::<Vec<PackageTemp>>().ok_or(CommonError::MissingResult)?;
        let tmp_path = context.get::<TmpPath>().ok_or(CommonError::MissingResult)?;
        let deploy = context.get::<Deploy>().ok_or(CommonError::MissingResult)?;

        let current_prefix = current_prefix_digest()?;
        let repository = deploy.open_repository()?;
        let mut tree = deploy.open_tree(&current_prefix)?;

        let database_bytes = FileHandle::new(DATABASE_PATH).read_file(&repository, &tree)?;
        let mut database = MemoryDatabase::open_in_memory(database_bytes)?;

        let mut config_defaults = FileSystem::new(Stat::uninitialized());
        let mut import_ctx = ImportContext::default();

        let mut package_data = ImportPackageData {
            repository: &repository,
            tree: &mut tree,
            config_defaults: &mut config_defaults,
            database: &mut database,
            import_ctx: &mut import_ctx,
            cancel,
        };

        let packages_total = packages.len() as u64;

        for (index, package) in packages.iter().enumerate() {
            if cancel.is_cancelled() {
                return Err(CommonError::Cancelled.into());
            }

            progress = progress
                .subject(package.meta.name.clone())
                .progress(index as u64, packages_total);
            context.send_progress(&progress);

            let trigger = context
                .get::<Vec<DeclarativeTrigger>>()
                .and_then(|triggers| triggers.get(index))
                .ok_or(CommonError::MissingResult)?;

            self.import_package(package, trigger, &mut package_data)?;
        }

        let database_bytes = database.into_bytes()?;
        let database_scratch_path = format!("{}/{INSTALLER_SCRATCH_FILENAME}", tmp_path.as_ref());
        write(&database_scratch_path, &database_bytes).map_err(RepoError::from)?;

        FileHandle::new(DATABASE_PATH).insert_file(
            &repository,
            &mut tree,
            &File::open(&database_scratch_path).map_err(RepoError::from)?,
            Stat::uninitialized(),
            &mut import_ctx,
        )?;

        let digest = commit_tree(&repository, tree)?;

        context.put(NewPrefixDigest(digest.to_hex()));
        context.put(NewConfigDefaults(config_defaults));

        Ok((progress, Box::new(NoRollback)))
    }
}

impl TransactionStage {
    fn import_package(
        &self, package: &PackageTemp, trigger: &DeclarativeTrigger, package_data: &mut ImportPackageData,
    ) -> Result<(), InstallError> {
        let source_root = Path::new(&package.temp_package_path);

        let usr_source = source_root.join("usr");
        let imported = if usr_source.is_dir() {
            FileHandle::new(PathBuf::new()).import_directory(
                package_data.repository,
                package_data.tree,
                &usr_source,
                package_data.import_ctx,
                package_data.cancel,
            )?
        } else {
            Vec::new()
        };

        let config_source = source_root.join("etc");
        let imported_config = if config_source.is_dir() {
            FileHandle::new(PathBuf::new()).import_directory(
                package_data.repository,
                package_data.config_defaults,
                &config_source,
                package_data.import_ctx,
                package_data.cancel,
            )?
        } else {
            Vec::new()
        };

        let uuid = package_data.database.insert_package_meta(&package.meta)?;
        package_data.database.set_declarative_triggers(uuid, trigger)?;

        for path in imported {
            let entry = FileEntry {
                path: path.to_string_lossy().into_owned(),
                is_user: false,
                scope: FileEntryScope::Prefix,
            };
            package_data.database.insert_package_file(uuid, &entry)?;
        }

        for path in imported_config {
            let entry = FileEntry {
                path: path.to_string_lossy().into_owned(),
                is_user: false,
                scope: FileEntryScope::Config,
            };
            package_data.database.insert_package_file(uuid, &entry)?;
        }

        Ok(())
    }
}
