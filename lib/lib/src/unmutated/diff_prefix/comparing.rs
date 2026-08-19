// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later

use upac_abi::hook::{CancelToken, ProgressEventBuilder};
use upac_abi::{DiffFileSource, FileDiffKind};

use crate::database::attribution::FileAttribute;
use crate::errors::CommonError;
use crate::orchestrator::Context;
use crate::orchestrator::stage::{NoRollback, RollbackGuard, Stage};
use crate::unmutated::diff_prefix::{DiffPrefixError, DiffPrefixSnapshot};

use upac_types::{DiffFileEntryCommon, DiffPrefixFileEntry};

pub struct ComparingStage;

impl Stage<DiffPrefixError> for ComparingStage {
    fn run(
        &self, context: &mut Context, _cancel: &CancelToken, progress: ProgressEventBuilder,
    ) -> Result<(ProgressEventBuilder, Box<dyn RollbackGuard>), DiffPrefixError> {
        let snapshot = context.take::<DiffPrefixSnapshot>().ok_or(CommonError::MissingResult)?;

        let mut entries = Vec::new();

        for (path, kind) in snapshot.changed {
            let database = match kind {
                FileDiffKind::Removed => &snapshot.from_database,
                FileDiffKind::Added | FileDiffKind::Modified => &snapshot.to_database,
            };

            if let Some(attribution) = database.attribute_file(&path)? {
                entries.push(DiffPrefixFileEntry {
                    common: DiffFileEntryCommon { path, kind },
                    source: DiffFileSource::Prefix,
                    package_name: attribution.package_meta.name,
                    is_user: attribution.file_entry.is_user,
                });
            }
        }

        context.put(entries);

        Ok((progress, Box::new(NoRollback)))
    }
}
