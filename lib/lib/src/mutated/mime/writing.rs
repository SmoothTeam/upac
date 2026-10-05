// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::path::Path;
use std::process::Command;

use upac_types::CancelToken;
use upac_types::error::ErrorKind;

use upac_composefs::fs::WrittenFile;

use upac_macro::stage;

use upac_orchestrator::context::Context;
use upac_orchestrator::stage::Stage;

use super::RenderedFiles;

use crate::layout::mime::{APPLICATIONS_DIR, MIME_DB_DIR, UPDATE_DESKTOP_DATABASE_BIN, UPDATE_MIME_DATABASE_BIN};

pub struct WritingStage;

#[stage]
impl Stage<ErrorKind> for WritingStage {
    fn run(
        &self, context: &mut Context, cancel: &CancelToken, progress: &dyn Fn(Option<&str>, u64, u64),
    ) -> Result<(), ErrorKind> {
        let rendered = context.take::<RenderedFiles>()?;
        let total = rendered.0.len() as u64;

        let mut written = Vec::new();
        for (position, (path, content)) in rendered.0.iter().enumerate() {
            let result = if cancel.is_cancelled() {
                Err(ErrorKind::Cancelled)
            } else {
                WrittenFile::write(Path::new(path), content.as_bytes()).map_err(ErrorKind::from)
            };

            match result {
                Ok(written_file) => written.push(written_file),
                Err(error) => {
                    for written_file in written.iter().rev() {
                        written_file.restore()?;
                    }
                    return Err(error);
                }
            }

            progress(Some(path), position as u64 + 1, total);
        }

        let _ = Command::new(UPDATE_MIME_DATABASE_BIN).arg(MIME_DB_DIR).status();
        let _ = Command::new(UPDATE_DESKTOP_DATABASE_BIN).arg(APPLICATIONS_DIR).status();

        Ok(())
    }
}
