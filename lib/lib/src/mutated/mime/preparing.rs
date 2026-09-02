// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::fs;

use upac_abi::hook::{CancelToken, ProgressEventBuilder};

use crate::errors::CommonError;
use crate::layout::{decoders, mime};
use crate::mutated::mime::{DesktopContent, MimeError};
use crate::orchestrator::Context;
use crate::orchestrator::stage::{NoRollback, RollbackGuard, Stage, StageResult};
use crate::plugin::decoder::manifest::load_decoder_manifests;

pub struct PreparingStage;

impl Stage<MimeError> for PreparingStage {
    fn run(
        &self, context: &mut Context, _cancel: &CancelToken, progress: ProgressEventBuilder,
    ) -> Result<(ProgressEventBuilder, Box<dyn RollbackGuard>), MimeError> {
        let manifests = load_decoder_manifests(decoders::DECODERS_DIR, decoders::MANIFEST_EXTENSION)
            .map_err(CommonError::Decoder)?;
        let desktop_content = fs::read_to_string(mime::DESKTOP_FILE_PATH)?;

        context.put(manifests);
        context.put(DesktopContent(desktop_content));

        Ok((progress, Box::new(NoRollback::new_none(StageResult::Advance))))
    }
}
