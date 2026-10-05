// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_types::error::ErrorKind;
use upac_types::request::mutated::RollbackRequest;
use upac_types::state::mutated::RollbackStateId;

use upac_composefs::Digest;
use upac_composefs::fs::WrittenFile;

use upac_deploy::{Sysroot, SysrootMode};

use upac_orchestrator::context::Context;
use upac_orchestrator::{OrchestratorRun, SequentialOrchestrator, stages};

use self::select::SelectStage;

use super::stages::RequestedBootPlugin;
use super::stages::checkout::CheckoutStage;
use super::stages::swap::SwapStage;

use crate::report_progress;

mod select;

pub(crate) struct RequestedConfig {
    pub config_digest: Digest,
    pub discard_etc_changes: bool,
}

pub(crate) struct SelectionWrites(pub Vec<WrittenFile>);

pub fn run(request: RollbackRequest<'_>) -> Result<(), (RollbackStateId, ErrorKind)> {
    let config_digest =
        Digest::from_hex(request.config_digest).map_err(|error| (RollbackStateId::Setup, error.into()))?;
    let sysroot = Sysroot::new(SysrootMode::ReadWrite).map_err(|error| (RollbackStateId::Setup, error.into()))?;

    let mut context = Context::default();
    context.put(sysroot);
    context.put(RequestedConfig {
        config_digest,
        discard_etc_changes: request.discard_etc_changes,
    });
    context.put(RequestedBootPlugin(request.boot_plugin.to_owned()));

    SequentialOrchestrator::new(stages![SelectStage, CheckoutStage, SwapStage]).run_mutating(
        &mut context,
        request.base.cancel_token,
        &|event| report_progress(&request.base, event),
    )
}
