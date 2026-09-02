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
use crate::orchestrator::Context;
use crate::orchestrator::stage::{NoRollback, RollbackGuard, Stage, StageResult};

pub struct RemovePackageStage;

impl Stage<UninstallError> for RemovePackageStage {
    fn run(
        &self, context: &mut Context, _cancel: &CancelToken, mut progress: ProgressEventBuilder,
    ) -> Result<(ProgressEventBuilder, StageResult, Box<dyn RollbackGuard>), UninstallError> {
        let mut pending = context.take::<PendingUuids>().ok_or(CommonError::MissingResult)?;
        let mut tree = context.take::<WorkingTree>().ok_or(CommonError::MissingResult)?;
        let mut database = context.take::<WorkingDatabase>().ok_or(CommonError::MissingResult)?;
        let mut removed_config_paths = context
            .take::<WorkingRemovedConfigPaths>()
            .ok_or(CommonError::MissingResult)?;
        let total = context.get::<TotalPackages>().ok_or(CommonError::MissingResult)?;
        let purge = context.get::<Purge>().ok_or(CommonError::MissingResult)?;

        let uuid = pending.0.pop_front().ok_or(CommonError::MissingResult)?;

        let subject = database
            .0
            .get_package_meta(uuid)?
            .map(|meta| meta.name)
            .unwrap_or_default();

        let files = database.0.list_package_files(uuid)?;

        for entry in files {
            if entry.is_user && !purge.0 {
                continue;
            }

            match entry.scope {
                FileEntryScope::Prefix => {
                    FileHandle::new(&entry.path).remove_in_tree(&mut tree.0)?;
                }
                FileEntryScope::Config => {
                    removed_config_paths.0.push(entry.path.clone());
                }
            }

            if entry.is_user {
                database.0.remove_user_file(uuid, &entry.path)?;
            } else {
                database.0.remove_package_file(uuid, &entry.path)?;
            }
        }

        let meta = database
            .0
            .get_package_meta(uuid)?
            .ok_or(UninstallError::PackageNotFound)?;
        database
            .0
            .remove_package_meta(&meta.name, &meta.arch, meta.arch_sub.as_deref())?;
        database.0.remove_declarative_triggers(uuid)?;

        let remaining = pending.0.len() as u64;
        let processed = total.0 - remaining;
        progress = progress.subject(subject).progress(processed, total.0);

        let result = if pending.0.is_empty() {
            StageResult::Advance
        } else {
            StageResult::Repeat
        };

        context.put(pending);
        context.put(tree);
        context.put(database);
        context.put(removed_config_paths);

        Ok((progress, result, Box::new(NoRollback)))
    }
}
