// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::collections::VecDeque;

use composefs::repository::ImportContext;

use upac_abi::hook::{CancelToken, ProgressEventBuilder};

use crate::composefs::file::FileHandle;
use crate::database::meta::MetaStore;
use crate::database::{InMemory, MemoryDatabase};
use crate::deploy::Deploy;
use crate::deploy::digest::current_prefix_digest;
use crate::errors::CommonError;
use crate::layout::database::DATABASE_PATH;
use crate::layout::deployment::ETC_UPPER_RELATIVE_PATH;
use crate::mutated::files::{
    EtcUpperDir, FilesError, PendingFiles, RequestedFilePackage, TargetUuid, TotalFiles, WorkingDatabase, WorkingTree,
};
use crate::orchestrator::Context;
use crate::orchestrator::stage::{NoRollback, RollbackGuard, Stage, StageResult};

pub struct OpenTransactionStage;

impl Stage<FilesError> for OpenTransactionStage {
    fn run(
        &self, context: &mut Context, _cancel: &CancelToken, progress: ProgressEventBuilder,
    ) -> Result<(ProgressEventBuilder, StageResult, Box<dyn RollbackGuard>), FilesError> {
        let files = context.take::<Vec<String>>().ok_or(CommonError::MissingResult)?;
        let file_package = context
            .get::<RequestedFilePackage>()
            .ok_or(CommonError::MissingResult)?;
        let deploy = context.get::<Deploy>().ok_or(CommonError::MissingResult)?;

        let current_prefix = current_prefix_digest()?;
        let repository = deploy.open_repository()?;
        let tree = deploy.open_tree(&current_prefix)?;

        let current_record_dir = deploy.deploy(&current_prefix);
        let etc_upper_dir = current_record_dir.join(ETC_UPPER_RELATIVE_PATH);

        let database_bytes = FileHandle::new(DATABASE_PATH).read_file(&repository, &tree)?;
        let database = MemoryDatabase::open_in_memory(database_bytes)?;

        let uuid = database
            .find_package_uuid(&file_package.name, &file_package.arch, file_package.arch_sub.as_deref())?
            .ok_or(FilesError::PackageNotFound)?;

        let total = files.len() as u64;
        let pending: VecDeque<_> = files.into_iter().collect();

        context.put(WorkingTree(tree));
        context.put(WorkingDatabase(database));
        context.put(ImportContext::default());
        context.put(EtcUpperDir(etc_upper_dir));
        context.put(TargetUuid(uuid));
        context.put(PendingFiles(pending));
        context.put(TotalFiles(total));

        Ok((progress, StageResult::Advance, Box::new(NoRollback)))
    }
}
