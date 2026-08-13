// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 JustPav
//
// SPDX-License-Identifier: LGPL-3.0-or-later

use upac_abi::DiffKind;
use upac_abi::hook::{CancelToken, ProgressEventBuilder};

use crate::database::MemoryDatabase;
use crate::database::files::FileStore;
use crate::database::meta::MetaStore;
use crate::errors::CommonError;
use crate::orchestrator::Context;
use crate::orchestrator::stage::{NoRollback, RollbackGuard, Stage};
use crate::types::DiffPrefixFileEntry;
use crate::unmutated::diff_files_prefix::{DiffFilesPrefixError, DiffFilesPrefixSnapshot};

pub struct ComparingStage;

impl Stage<DiffFilesPrefixError> for ComparingStage {
    fn run(
        &self, context: &mut Context, _cancel: &CancelToken, progress: ProgressEventBuilder,
    ) -> Result<(ProgressEventBuilder, Box<dyn RollbackGuard>), DiffFilesPrefixError> {
        let snapshot = context
            .take::<DiffFilesPrefixSnapshot>()
            .ok_or(CommonError::MissingResult)?;

        let mut entries = Vec::new();

        for (path, kind) in snapshot.changed {
            let database = match kind {
                DiffKind::Removed => &snapshot.from_database,
                DiffKind::Added | DiffKind::Modified => &snapshot.to_database,
            };

            if let Some(entry) = Self::attribute(database, &path, kind)? {
                entries.push(entry);
            }
        }

        context.put(entries);

        Ok((progress, Box::new(NoRollback)))
    }
}

impl ComparingStage {
    fn attribute(
        database: &MemoryDatabase, path: &str, kind: DiffKind,
    ) -> Result<Option<DiffPrefixFileEntry>, DiffFilesPrefixError> {
        let Some(uuid) = database.find_file_owner(path)? else {
            return Ok(None);
        };
        let Some(meta) = database.get_package_meta(uuid)? else {
            return Ok(None);
        };
        let is_user = database
            .list_files(uuid)?
            .into_iter()
            .find(|entry| entry.path == path)
            .is_some_and(|entry| entry.is_user);

        Ok(Some(DiffPrefixFileEntry {
            path: path.to_owned(),
            kind,
            package_name: meta.name,
            is_user,
        }))
    }
}
