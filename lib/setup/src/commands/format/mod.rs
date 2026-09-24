// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::path::PathBuf;

use upac::errors::CommonError;
use upac::orchestrator::context::Context;
use upac::orchestrator::{Orchestrator, SequentialOrchestrator, run_mutating, stages};

use upac_abi::FsKind;

use upac_types::request::format::SetupFormatRequest;
use upac_types::state::setup::FormatStateId;

use upac_macro::ContextValue;

use self::error::FormatError;
use self::mkfs::MkfsStage;
use self::verify::VerifyStage;
use self::wipe::WipeStage;

pub mod error;

mod filesystem;
mod mkfs;
mod verify;
mod wipe;

#[derive(ContextValue)]
pub(crate) struct RequestedDevice(pub PathBuf);

#[derive(ContextValue)]
pub(crate) struct RequireEsp(pub bool);

#[derive(ContextValue)]
pub(crate) struct ForceWipe(pub bool);

pub(crate) struct RequestedFilesystem {
    pub fs_kind: FsKind,
    pub label: Option<String>,
    pub btrfs_node_size: u32,
    pub btrfs_sector_size: u32,
}

pub fn run(request: SetupFormatRequest<'_>) -> Result<(), (FormatStateId, FormatError)> {
    let cancel_token = unsafe { request.base.cancel_token.as_ref() }
        .ok_or((FormatStateId::Setup, FormatError::from(CommonError::PipelineInvalid)))?;

    let mut context = Context::new();
    context.put(request.base.message_hook());
    context.put(RequestedDevice(PathBuf::from(request.device_path)));
    context.put(RequireEsp(request.require_esp));
    context.put(ForceWipe(request.force_wipe));
    context.put(RequestedFilesystem {
        fs_kind: request.fs_kind,
        label: request.label.map(str::to_owned),
        btrfs_node_size: request.btrfs_node_size,
        btrfs_sector_size: request.btrfs_sector_size,
    });

    let orchestrator = SequentialOrchestrator::new(stages![VerifyStage, WipeStage, MkfsStage]);

    let result = run_mutating!(orchestrator, context, cancel_token, FormatStateId, FormatError);

    cancel_token.reset();

    result
}
