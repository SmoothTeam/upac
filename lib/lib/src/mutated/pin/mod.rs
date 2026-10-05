// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_types::error::ErrorKind;
use upac_types::request::mutated::PinRequest;
use upac_types::state::mutated::PinStateId;

use upac_composefs::Digest;

use upac_deploy::{Sysroot, SysrootMode};

use upac_orchestrator::context::Context;
use upac_orchestrator::{OrchestratorRun, SequentialOrchestrator, stages};

use self::stage::SetPinnedStage;

use crate::report_progress;

mod stage;

pub(crate) struct RequestedPrefixDigest(pub Digest);

pub(crate) struct RequestedPinned(pub bool);

pub fn run(request: PinRequest<'_>) -> Result<(), (PinStateId, ErrorKind)> {
    let prefix_digest = Digest::from_hex(request.prefix_digest).map_err(|error| (PinStateId::Setup, error.into()))?;
    let sysroot = Sysroot::new(SysrootMode::ReadWrite).map_err(|error| (PinStateId::Setup, error.into()))?;

    let mut context = Context::default();
    context.put(sysroot);
    context.put(RequestedPrefixDigest(prefix_digest));
    context.put(RequestedPinned(request.pinned));

    SequentialOrchestrator::new(stages![SetPinnedStage]).run_mutating(
        &mut context,
        request.base.cancel_token,
        &|event| report_progress(&request.base, event),
    )
}
