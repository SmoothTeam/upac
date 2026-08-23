// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later

use std::fs::{File, write};
use std::path::{Path, PathBuf};

use composefs::fsverity::FsVerityHashValue;
use composefs::generic_tree::Stat;
use composefs::repository::{ImportContext, Repository};
use composefs::tree::FileSystem;

use upac_abi::hook::{CancelToken, ProgressEventBuilder};

use upac_types::{FileEntry, FileEntryScope, PackageTemp, TmpPath};

use crate::composefs::error::RepoError;
use crate::composefs::file::FileHandle;
use crate::composefs::repository::{ObjectID, commit_tree};
use crate::database::files::{FileStore, FileStoreMut};
use crate::database::meta::{MetaStore, MetaStoreMut};
use crate::database::{InMemory, MemoryDatabase};
use crate::deploy::Deploy;
use crate::deploy::digest::current_prefix_digest;
use crate::errors::CommonError;
use crate::layout::database::DATABASE_PATH;
use crate::mutated::update::{AllowDowngrade, NewConfigDefaults, NewPrefixDigest, RemovedConfigPaths, UpdateError};
use crate::orchestrator::Context;
use crate::orchestrator::stage::{NoRollback, RollbackGuard, Stage};

pub struct TransactionStage;

struct UpdatePackageData<'a> {
    repository: &'a Repository<ObjectID>,
    tree: &'a mut FileSystem<ObjectID>,
    config_defaults: &'a mut FileSystem<ObjectID>,
    removed_config_paths: &'a mut Vec<String>,
    database: &'a mut MemoryDatabase,
    import_ctx: &'a mut ImportContext,
    allow_downgrade: bool,
}

impl Stage<UpdateError> for TransactionStage {
    fn run(
        &self, context: &mut Context, _cancel: &CancelToken, progress: ProgressEventBuilder,
    ) -> Result<(ProgressEventBuilder, Box<dyn RollbackGuard>), UpdateError> {
        let packages = context.take::<Vec<PackageTemp>>().ok_or(CommonError::MissingResult)?;
        let tmp_path = context.get::<TmpPath>().ok_or(CommonError::MissingResult)?;
        let deploy = context.get::<Deploy>().ok_or(CommonError::MissingResult)?;
        let allow_downgrade = context.get::<AllowDowngrade>().ok_or(CommonError::MissingResult)?;

        let current_prefix = current_prefix_digest()?;
        let repository = deploy.open_repository()?;
        let mut tree = deploy.open_tree(&current_prefix)?;

        let database_bytes = FileHandle::new(DATABASE_PATH).read_file(&repository, &tree)?;
        let mut database = MemoryDatabase::open_in_memory(database_bytes)?;

        let mut config_defaults = FileSystem::new(Stat::uninitialized());
        let mut removed_config_paths = Vec::new();
        let mut import_ctx = ImportContext::default();

        let mut package_data = UpdatePackageData {
            repository: &repository,
            tree: &mut tree,
            config_defaults: &mut config_defaults,
            removed_config_paths: &mut removed_config_paths,
            database: &mut database,
            import_ctx: &mut import_ctx,
            allow_downgrade: allow_downgrade.0,
        };

        for package in &packages {
            self.update_package(package, &mut package_data)?;
        }

        let database_bytes = database.into_bytes()?;
        let database_scratch_path = format!("{}/update-packages.redb", tmp_path.as_ref());
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
        context.put(RemovedConfigPaths(removed_config_paths));

        Ok((progress, Box::new(NoRollback)))
    }
}

impl TransactionStage {
    fn update_package(&self, package: &PackageTemp, package_data: &mut UpdatePackageData) -> Result<(), UpdateError> {
        let uuid = package_data
            .database
            .find_package_uuid(&package.meta.name, &package.meta.arch, package.meta.arch_sub.as_deref())?
            .ok_or(UpdateError::PackageNotFound)?;

        if !package_data.allow_downgrade {
            let current_meta = package_data
                .database
                .get_package_meta(uuid)?
                .ok_or(UpdateError::PackageNotFound)?;

            if package.meta.version < current_meta.version {
                return Err(UpdateError::DowngradeNotAllowed);
            }
        }

        for entry in package_data.database.list_package_files(uuid)? {
            match entry.scope {
                FileEntryScope::Prefix => {
                    FileHandle::new(&entry.path).remove_in_tree(package_data.tree)?;
                }
                FileEntryScope::Config => {
                    package_data.removed_config_paths.push(entry.path.clone());
                }
            }

            package_data.database.remove_package_file(uuid, &entry.path)?;
        }

        let source_root = Path::new(&package.temp_package_path);

        let usr_source = source_root.join("usr");
        let imported = if usr_source.is_dir() {
            FileHandle::new(PathBuf::new()).import_directory(
                package_data.repository,
                package_data.tree,
                &usr_source,
                package_data.import_ctx,
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
            )?
        } else {
            Vec::new()
        };

        package_data.database.update_package_meta(&package.meta)?;

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
