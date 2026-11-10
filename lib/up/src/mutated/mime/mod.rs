// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_types::error::ErrorKind;
use upac_types::request::mutated::MimeSyncRequest;
use upac_types::state::mutated::MimeStateId;

use upac_orchestrator::context::Context;
use upac_orchestrator::{OrchestratorRun, SequentialOrchestrator, stages};

use self::preparing::PreparingStage;
use self::rendering::RenderingStage;
use self::writing::WritingStage;

mod preparing;
mod rendering;
mod writing;

pub(crate) struct DesktopContent(pub String);

pub(crate) struct RenderedFiles(pub Vec<(&'static str, String)>);

pub fn run(request: MimeSyncRequest<'_>) -> Result<(), (MimeStateId, ErrorKind, Option<String>)> {
    let mut context = Context::default();

    SequentialOrchestrator::new(stages![PreparingStage, RenderingStage, WritingStage]).run_mutating(
        &mut context,
        request.base.cancel_token,
        &|event| request.base.report_progress(event),
    )
}
