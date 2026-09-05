// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::path::Path;

use composefs::repository::ImportContext;

use upac_abi::hook::{CancelToken, ProgressEventBuilder};

use upac_types::{FileEntry, FileEntryScope};

use crate::composefs::file::import_if_dir;
use crate::database::files::FileStoreMut;
use crate::database::meta::MetaStoreMut;
use crate::database::triggers::TriggerStoreMut;
use crate::deploy::Deploy;
use crate::errors::CommonError;
use crate::mutated::installer::{
    ImportedConfigDefaults, ImportedDatabase, ImportedTree, InstallError, PendingPackages, TotalPackages,
};
use crate::orchestrator::stage::{NoRollback, RollbackGuard, Stage, StageResult};
use crate::orchestrator::{Context, ctx_get, ctx_take};

pub struct ImportPackageStage;

impl Stage<InstallError> for ImportPackageStage {
    fn run(
        &self, context: &mut Context, cancel: &CancelToken, mut progress: ProgressEventBuilder,
    ) -> Result<(ProgressEventBuilder, StageResult, Box<dyn RollbackGuard>), InstallError> {
        let mut pending_packages = ctx_take!(context, PendingPackages);
        let mut imported_tree = ctx_take!(context, ImportedTree);
        let mut config_defaults = ctx_take!(context, ImportedConfigDefaults);
        let mut database = ctx_take!(context, ImportedDatabase);
        let mut import_ctx = ctx_take!(context, ImportContext);

        let total = ctx_get!(context, TotalPackages);
        let deploy = ctx_get!(context, Deploy);

        let (package, trigger) = pending_packages.0.pop_front().ok_or(CommonError::MissingResult)?;

        let repository = deploy.open_repository()?;
        let source_root = Path::new(&package.temp_package_path);

        let prefix_source = source_root.join("usr");
        let imported = import_if_dir!(
            &repository,
            &mut imported_tree.0,
            &prefix_source,
            &mut import_ctx,
            cancel
        );

        let config_source = source_root.join("etc");
        let imported_config = import_if_dir!(
            &repository,
            &mut config_defaults.0,
            &config_source,
            &mut import_ctx,
            cancel
        );

        let uuid = database.0.insert_package_meta(&package.meta)?;
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

        let remaining = pending_packages.0.len() as u64;
        let processed = total.0 - remaining;
        progress = progress.subject(package.meta.name.clone()).progress(processed, total.0);

        let stage_result = if pending_packages.0.is_empty() {
            StageResult::Advance
        } else {
            StageResult::Repeat
        };

        context.put(pending_packages);
        context.put(imported_tree);
        context.put(config_defaults);
        context.put(database);
        context.put(import_ctx);

        Ok((progress, stage_result, Box::new(NoRollback)))
    }
}
