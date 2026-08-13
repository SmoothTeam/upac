// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 JustPav
//
// SPDX-License-Identifier: LGPL-3.0-or-later

use upac_abi::FileDiffKind;
use upac_abi::hook::{CancelToken, ProgressEventBuilder};

use crate::database::MemoryDatabase;
use crate::database::files::FileStore;
use crate::database::meta::MetaStore;
use crate::errors::CommonError;
use crate::orchestrator::Context;
use crate::orchestrator::stage::{NoRollback, RollbackGuard, Stage};
use crate::types::DiffConfigFileEntry;
use crate::unmutated::diff_files_config::{DiffFilesConfigError, DiffFilesConfigSnapshot};

pub struct ComparingStage;

impl Stage<DiffFilesConfigError> for ComparingStage {
    fn run(
        &self, context: &mut Context, _cancel: &CancelToken, progress: ProgressEventBuilder,
    ) -> Result<(ProgressEventBuilder, Box<dyn RollbackGuard>), DiffFilesConfigError> {
        let snapshot = context
            .take::<DiffFilesConfigSnapshot>()
            .ok_or(CommonError::MissingResult)?;

        let mut entries = Vec::new();

        for (path, kind) in snapshot.changed {
            let database = match kind {
                FileDiffKind::Removed => &snapshot.from_database,
                FileDiffKind::Added | FileDiffKind::Modified => &snapshot.to_database,
            };

            let package_name = Self::attribute(database, &path)?;

            entries.push(DiffConfigFileEntry {
                path,
                kind,
                package_name,
            });
        }

        context.put(entries);

        Ok((progress, Box::new(NoRollback)))
    }
}

impl ComparingStage {
    fn attribute(database: &MemoryDatabase, path: &str) -> Result<Option<String>, DiffFilesConfigError> {
        let Some(uuid) = database.find_file_owner(path)? else {
            return Ok(None);
        };
        let Some(meta) = database.get_package_meta(uuid)? else {
            return Ok(None);
        };

        Ok(Some(meta.name))
    }
}
