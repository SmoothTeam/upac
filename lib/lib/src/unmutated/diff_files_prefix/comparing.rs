// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 JustPav
//
// SPDX-License-Identifier: LGPL-3.0-or-later

use upac_abi::FileDiffKind;
use upac_abi::hook::{CancelToken, ProgressEventBuilder};

use crate::database::attribution::FileAttribute;
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
                FileDiffKind::Removed => &snapshot.from_database,
                FileDiffKind::Added | FileDiffKind::Modified => &snapshot.to_database,
            };

            if let Some(attribution) = database.attribute_file(&path)? {
                entries.push(DiffPrefixFileEntry {
                    path,
                    kind,
                    package_name: attribution.package_meta.name,
                    is_user: attribution.file_entry.is_user,
                });
            }
        }

        context.put(entries);

        Ok((progress, Box::new(NoRollback)))
    }
}
