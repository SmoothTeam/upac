// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::path::Path;
use std::process::Command;

use upac_abi::hook::{CancelToken, ProgressEventBuilder};

use crate::errors::CommonError;
use crate::fs::WrittenFile;
use crate::layout::mime;
use crate::mutated::mime::{MimeError, PendingWrites, TotalWrites};
use crate::orchestrator::stage::{RollbackGuard, Stage, StageResult};
use crate::orchestrator::{Context, ctx_get, ctx_take};

pub struct WritingStage;

impl Stage<MimeError> for WritingStage {
    fn run(
        &self, context: &mut Context, _cancel: &CancelToken, mut progress: ProgressEventBuilder,
    ) -> Result<(ProgressEventBuilder, StageResult, Box<dyn RollbackGuard>), MimeError> {
        let mut pending = ctx_take!(context, PendingWrites);

        let total = ctx_get!(context, TotalWrites);

        let (path, content) = pending.0.pop_front().ok_or(CommonError::MissingResult)?;

        let written_file = WrittenFile::write(Path::new(path), content.as_bytes())?;

        let remaining = pending.0.len() as u64;
        let processed = total.0 - remaining;
        progress = progress.subject(path.to_owned()).progress(processed, total.0);

        let result = if pending.0.is_empty() {
            let _ = Command::new(mime::UPDATE_MIME_DATABASE_BIN)
                .arg(mime::MIME_DB_DIR)
                .status();
            let _ = Command::new(mime::UPDATE_DESKTOP_DATABASE_BIN)
                .arg(mime::APPLICATIONS_DIR)
                .status();

            StageResult::Advance
        } else {
            StageResult::Repeat
        };

        context.put(pending);

        Ok((progress, result, Box::new(vec![written_file])))
    }
}
