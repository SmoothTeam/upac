// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac::database::files::FileStoreMut;
use upac::errors::CommonError;
use upac::orchestrator::Context;
use upac::orchestrator::stage::{RollbackGuard, Stage, StageResult};

use upac_abi::error::ErrorKind;
use upac_abi::hook::{CancelToken, ProgressEventBuilder};

use upac_types::{FileEntry, FileEntryScope};

use crate::error::SetupError;
use crate::genesis::{GenesisDatabase, ImportedConfigPaths, ImportedPrefixPaths, PackageUuid};

pub struct InsertFileEntryStage;

struct InsertFileEntryGuard(StageResult);

impl RollbackGuard for InsertFileEntryGuard {
    fn new_none(result: StageResult) -> Self {
        InsertFileEntryGuard(result)
    }

    fn rollback(&mut self) -> Result<(), ErrorKind> {
        Ok(())
    }

    fn result(&self) -> StageResult {
        self.0
    }
}

impl Stage<SetupError> for InsertFileEntryStage {
    fn run(
        &self, context: &mut Context, _cancel: &CancelToken, progress: ProgressEventBuilder,
    ) -> Result<(ProgressEventBuilder, Box<dyn RollbackGuard>), SetupError> {
        let mut prefix_paths = context
            .take::<ImportedPrefixPaths>()
            .ok_or(CommonError::MissingResult)?;
        let mut config_paths = context
            .take::<ImportedConfigPaths>()
            .ok_or(CommonError::MissingResult)?;
        let mut database = context.take::<GenesisDatabase>().ok_or(CommonError::MissingResult)?;
        let uuid = context.get::<PackageUuid>().ok_or(CommonError::MissingResult)?;

        let next = if let Some(path) = prefix_paths.0.pop() {
            Some((FileEntryScope::Prefix, path))
        } else {
            config_paths.0.pop().map(|path| (FileEntryScope::Config, path))
        };

        if let Some((scope, path)) = next {
            database.0.insert_package_file(
                uuid.0,
                &FileEntry {
                    path: path.to_string_lossy().into_owned(),
                    is_user: false,
                    scope,
                },
            )?;
        }

        let done = prefix_paths.0.is_empty() && config_paths.0.is_empty();

        context.put(prefix_paths);
        context.put(config_paths);
        context.put(database);

        let result = if done {
            StageResult::Advance
        } else {
            StageResult::Repeat
        };

        Ok((progress, Box::new(InsertFileEntryGuard(result))))
    }
}
