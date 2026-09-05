// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_abi::hook::{CancelToken, ProgressEventBuilder};

use upac_types::FileEntryScope;

use crate::composefs::file::FileHandle;
use crate::database::files::{FileStore, FileStoreMut};
use crate::database::meta::{MetaStore, MetaStoreMut};
use crate::database::triggers::TriggerStoreMut;
use crate::errors::CommonError;
use crate::mutated::uninstaller::{
    PendingUuids, Purge, TotalPackages, UninstallError, WorkingDatabase, WorkingRemovedConfigPaths, WorkingTree,
};
use crate::orchestrator::stage::{NoRollback, RollbackGuard, Stage, StageResult};
use crate::orchestrator::{Context, ctx_get, ctx_take};

pub struct RemovePackageStage;

impl Stage<UninstallError> for RemovePackageStage {
    fn run(
        &self, context: &mut Context, _cancel: &CancelToken, mut progress: ProgressEventBuilder,
    ) -> Result<(ProgressEventBuilder, StageResult, Box<dyn RollbackGuard>), UninstallError> {
        let mut pending = ctx_take!(context, PendingUuids);
        let mut woking_tree = ctx_take!(context, WorkingTree);
        let mut woking_database = ctx_take!(context, WorkingDatabase);
        let mut removed_config_paths = ctx_take!(context, WorkingRemovedConfigPaths);

        let total_packages = ctx_get!(context, TotalPackages);
        let purge = ctx_get!(context, Purge);

        let uuid = pending.0.pop_front().ok_or(CommonError::MissingResult)?;

        let subject = woking_database
            .0
            .get_package_meta(uuid)?
            .map(|meta| meta.name)
            .unwrap_or_default();

        let files = woking_database.0.list_package_files(uuid)?;

        for entry in files {
            if entry.is_user && !purge.0 {
                continue;
            }

            match entry.scope {
                FileEntryScope::Prefix => {
                    FileHandle::new(&entry.path).remove_in_tree(&mut woking_tree.0)?;
                }
                FileEntryScope::Config => {
                    removed_config_paths.0.push(entry.path.clone());
                }
            }

            if entry.is_user {
                woking_database.0.remove_user_file(uuid, &entry.path)?;
            } else {
                woking_database.0.remove_package_file(uuid, &entry.path)?;
            }
        }

        let meta = woking_database
            .0
            .get_package_meta(uuid)?
            .ok_or(UninstallError::PackageNotFound)?;
        woking_database
            .0
            .remove_package_meta(&meta.name, &meta.arch, meta.arch_sub.as_deref())?;
        woking_database.0.remove_declarative_triggers(uuid)?;

        let remaining = pending.0.len() as u64;
        let processed = total_packages.0 - remaining;
        progress = progress.subject(subject).progress(processed, total_packages.0);

        let result = if pending.0.is_empty() {
            StageResult::Advance
        } else {
            StageResult::Repeat
        };

        context.put(pending);
        context.put(woking_tree);
        context.put(woking_database);
        context.put(removed_config_paths);

        Ok((progress, result, Box::new(NoRollback)))
    }
}
