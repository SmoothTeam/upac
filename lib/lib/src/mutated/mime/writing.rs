// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later

use std::path::Path;
use std::process::Command;

use upac_abi::hook::{CancelToken, ProgressEventBuilder};

use crate::errors::CommonError;
use crate::fs::WrittenFile;
use crate::layout::mime;
use crate::mutated::mime::{MimeError, RenderedMime};
use crate::orchestrator::Context;
use crate::orchestrator::stage::{RollbackGuard, Stage};

pub struct WritingStage;

impl Stage<MimeError> for WritingStage {
    fn run(
        &self, context: &mut Context, _cancel: &CancelToken, progress: ProgressEventBuilder,
    ) -> Result<(ProgressEventBuilder, Box<dyn RollbackGuard>), MimeError> {
        let rendered = context.take::<RenderedMime>().ok_or(CommonError::MissingResult)?;

        let mut written: Vec<WrittenFile> = Vec::with_capacity(2);

        for (path, content) in [
            (mime::MIME_XML_PATH, rendered.mime_xml.as_bytes()),
            (mime::DESKTOP_FILE_PATH, rendered.desktop_content.as_bytes()),
        ] {
            match WrittenFile::write(Path::new(path), content) {
                Ok(file) => written.push(file),
                Err(error) => {
                    let _ = written.rollback();
                    return Err(error.into());
                }
            }
        }

        let _ = Command::new(mime::UPDATE_MIME_DATABASE_BIN)
            .arg(mime::MIME_DB_DIR)
            .status();
        let _ = Command::new(mime::UPDATE_DESKTOP_DATABASE_BIN)
            .arg(mime::APPLICATIONS_DIR)
            .status();

        Ok((progress, Box::new(written)))
    }
}
