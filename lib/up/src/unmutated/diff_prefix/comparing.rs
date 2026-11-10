// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_types::CancelToken;
use upac_types::diff::{DiffFileSource, FileDiffKind};
use upac_types::error::ErrorKind;
use upac_types::response::entry::{DiffFileCommonEntry, DiffPrefixFileEntry};
use upac_types::response::unmutated::DiffPrefixResponse;

use upac_database::attribution::FileAttribute;

use upac_macro::stage;

use upac_orchestrator::context::Context;
use upac_orchestrator::stage::Stage;

use super::DiffPrefixSnapshot;

pub struct ComparingStage;

#[stage]
impl Stage<ErrorKind> for ComparingStage {
    fn run(
        &self, context: &mut Context, _cancel: &CancelToken, _progress: &dyn Fn(Option<&str>, u64, u64),
    ) -> Result<(), ErrorKind> {
        let snapshot = context.take::<DiffPrefixSnapshot>()?;
        let mut files = Vec::new();

        for (path, kind) in snapshot.changed {
            let database = match kind {
                FileDiffKind::Removed => &snapshot.from_database,
                FileDiffKind::Added | FileDiffKind::Modified => &snapshot.to_database,
            };

            let path = path.to_string_lossy().into_owned();
            if let Some(attribution) = database.attribute_file(&path)? {
                files.push(DiffPrefixFileEntry {
                    common: DiffFileCommonEntry { path, kind },
                    source: DiffFileSource::Prefix,
                    package_name: attribution.package_meta.name,
                    is_user: attribution.file_entry.is_user,
                });
            }
        }

        context.put(DiffPrefixResponse { files });

        Ok(())
    }
}
