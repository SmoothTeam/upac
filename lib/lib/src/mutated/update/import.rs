// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::path::Path;

use composefs::repository::ImportContext;

use upac_abi::hook::{CancelToken, ProgressEventBuilder};

use upac_types::{FileEntry, FileEntryScope};

use crate::composefs::file::{FileHandle, import_if_dir};
use crate::database::files::{FileStore, FileStoreMut};
use crate::database::meta::{MetaStore, MetaStoreMut};
use crate::database::triggers::TriggerStoreMut;
use crate::deploy::Deploy;
use crate::errors::CommonError;
use crate::mutated::update::{
    AllowDowngrade, ImportedConfigDefaults, ImportedDatabase, ImportedRemovedConfigPaths, ImportedTree,
    PendingPackages, TotalPackages, UpdateError,
};
use crate::orchestrator::stage::{NoRollback, RollbackGuard, Stage, StageResult};
use crate::orchestrator::{Context, ctx_get, ctx_take};

pub struct ImportPackageStage;

impl Stage<UpdateError> for ImportPackageStage {
    fn run(
        &self, context: &mut Context, cancel: &CancelToken, mut progress: ProgressEventBuilder,
    ) -> Result<(ProgressEventBuilder, StageResult, Box<dyn RollbackGuard>), UpdateError> {
        let mut pending = ctx_take!(context, PendingPackages);
        let mut tree = ctx_take!(context, ImportedTree);
        let mut config_defaults = ctx_take!(context, ImportedConfigDefaults);
        let mut database = ctx_take!(context, ImportedDatabase);
        let mut removed_config_paths = ctx_take!(context, ImportedRemovedConfigPaths);
        let mut import_ctx = ctx_take!(context, ImportContext);

        let total = ctx_get!(context, TotalPackages);
        let allow_downgrade = ctx_get!(context, AllowDowngrade);
        let deploy = ctx_get!(context, Deploy);

        let (package, trigger) = pending.0.pop_front().ok_or(CommonError::MissingResult)?;

        let repository = deploy.open_repository()?;

        let uuid = database
            .0
            .find_package_uuid(&package.meta.name, &package.meta.arch, package.meta.arch_sub.as_deref())?
            .ok_or(UpdateError::PackageNotFound)?;

        if !allow_downgrade.0 {
            let current_meta = database.0.get_package_meta(uuid)?.ok_or(UpdateError::PackageNotFound)?;

            if package.meta.version < current_meta.version {
                return Err(UpdateError::DowngradeNotAllowed);
            }
        }

        let old_files = database.0.list_package_files(uuid)?;

        for entry in old_files {
            match entry.scope {
                FileEntryScope::Prefix => {
                    FileHandle::new(&entry.path).remove_in_tree(&mut tree.0)?;
                }
                FileEntryScope::Config => {
                    removed_config_paths.0.push(entry.path.clone());
                }
            }

            database.0.remove_package_file(uuid, &entry.path)?;
        }

        let source_root = Path::new(&package.temp_package_path);

        let usr_source = source_root.join("usr");
        let imported = import_if_dir!(&repository, &mut tree.0, &usr_source, &mut import_ctx, cancel);

        let config_source = source_root.join("etc");
        let imported_config = import_if_dir!(
            &repository,
            &mut config_defaults.0,
            &config_source,
            &mut import_ctx,
            cancel
        );

        database.0.update_package_meta(&package.meta)?;
        database.0.set_declarative_triggers(uuid, &trigger)?;

        for path in imported {
            database.0.insert_package_file(
                uuid,
                &FileEntry {
                    path: path.to_string_lossy().into_owned(),
                    is_user: false,
                    scope: FileEntryScope::Prefix,
                },
            )?;
        }

        for path in imported_config {
            database.0.insert_package_file(
                uuid,
                &FileEntry {
                    path: path.to_string_lossy().into_owned(),
                    is_user: false,
                    scope: FileEntryScope::Config,
                },
            )?;
        }

        let remaining = pending.0.len() as u64;
        let processed = total.0 - remaining;
        progress = progress.subject(package.meta.name.clone()).progress(processed, total.0);

        let result = if pending.0.is_empty() {
            StageResult::Advance
        } else {
            StageResult::Repeat
        };

        context.put(pending);
        context.put(tree);
        context.put(config_defaults);
        context.put(database);
        context.put(removed_config_paths);
        context.put(import_ctx);

        Ok((progress, result, Box::new(NoRollback)))
    }
}
