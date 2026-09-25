// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::fs;

use upac_abi::hook::CancelToken;

use upac_types::hook::ProgressEventBuilder;

use upac_decoder_loader::manifest::DecoderManifests;

use upac_orchestrator::context::Context;
use upac_orchestrator::stage::{NoRollback, RollbackGuard, Stage, StageResult};

use super::{DesktopContent, MimeError};

use crate::errors::CommonError;
use crate::layout::mime;

pub struct PreparingStage;

impl Stage<MimeError> for PreparingStage {
    fn run(
        &self, context: &mut Context, _cancel: &CancelToken, progress: ProgressEventBuilder,
    ) -> Result<(ProgressEventBuilder, StageResult, Box<dyn RollbackGuard>), MimeError> {
        let manifests = DecoderManifests::new().map_err(CommonError::Decoder)?;
        let desktop_content = fs::read_to_string(mime::DESKTOP_FILE_PATH)?;

        context.put(manifests);
        context.put(DesktopContent(desktop_content));

        Ok((progress, StageResult::Advance, Box::new(NoRollback)))
    }
}
