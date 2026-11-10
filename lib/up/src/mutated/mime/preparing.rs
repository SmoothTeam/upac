// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::fs::read_to_string;

use upac_types::CancelToken;
use upac_types::error::ErrorKind;

use upac_decoder_loader::manifest::DecoderManifests;

use upac_macro::stage;

use upac_orchestrator::context::Context;
use upac_orchestrator::stage::Stage;

use super::DesktopContent;

use crate::layout::mime::DESKTOP_FILE_PATH;

pub struct PreparingStage;

#[stage]
impl Stage<ErrorKind> for PreparingStage {
    fn run(
        &self, context: &mut Context, _cancel: &CancelToken, _progress: &dyn Fn(Option<&str>, u64, u64),
    ) -> Result<(), ErrorKind> {
        let manifests = DecoderManifests::new()?;
        let desktop_content = read_to_string(DESKTOP_FILE_PATH)?;

        context.put::<DecoderManifests>(manifests);
        context.put(DesktopContent(desktop_content));

        Ok(())
    }
}
