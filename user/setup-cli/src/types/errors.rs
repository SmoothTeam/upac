// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: GPL-3.0-only

use std::error::Error;
use std::fmt::{Display, Formatter, Result as FmtResult};

use i18n_embed_fl::fl;

use upac_types::error::{Error as AbiError, ErrorDomain, ErrorKind};
use upac_types::state::setup::{
    BootstrapDeployStateId, BootstrapImportStateId, BootstrapKernelStateId, FormatStateId, PartitionAddStateId,
    PartitionTableStateId,
};

use crate::locale::LOADER;

#[cfg(test)]
#[path = "../../tests/inline/errors.rs"]
mod tests;

#[derive(Debug)]
pub struct AbiMismatch {
    pub got: u32,
    pub expected: u32,
}

impl Display for AbiMismatch {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FmtResult {
        write!(
            formatter,
            "{} ({} → {})",
            fl!(LOADER, "abi-version-mismatch"),
            self.got,
            self.expected
        )
    }
}

impl Error for AbiMismatch {}

#[derive(Debug)]
pub struct InvalidResponse {
    pub error: ErrorKind,
}

impl Display for InvalidResponse {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FmtResult {
        write!(formatter, "{}", error_kind_message(self.error))
    }
}

impl Error for InvalidResponse {}

pub(crate) struct StageName {
    domain: ErrorDomain,
    state: u32,
}

impl StageName {
    pub(crate) fn new(domain: ErrorDomain, state: u32) -> Self {
        StageName { domain, state }
    }
}

impl Display for StageName {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FmtResult {
        let state = self.state as usize;

        let key = match self.domain {
            ErrorDomain::PartitionTable => PartitionTableStateId::from_stage_index(state).stage_key(),
            ErrorDomain::PartitionAdd => PartitionAddStateId::from_stage_index(state).stage_key(),
            ErrorDomain::Format => FormatStateId::from_stage_index(state).stage_key(),
            ErrorDomain::BootstrapImport => BootstrapImportStateId::from_stage_index(state).stage_key(),
            ErrorDomain::BootstrapKernel => BootstrapKernelStateId::from_stage_index(state).stage_key(),
            ErrorDomain::BootstrapDeploy => BootstrapDeployStateId::from_stage_index(state).stage_key(),
            _ => "stage-setup",
        };

        write!(formatter, "{}", LOADER.get(key))
    }
}

fn error_kind_message(kind: ErrorKind) -> String {
    match kind {
        ErrorKind::Unexpected => fl!(LOADER, "err-unexpected"),
        ErrorKind::OutOfMemory => fl!(LOADER, "err-oom"),
        ErrorKind::NotFound => fl!(LOADER, "err-not-found"),
        ErrorKind::AlreadyExists => fl!(LOADER, "err-already-exists"),
        ErrorKind::PermissionDenied => fl!(LOADER, "err-permission-denied"),
        ErrorKind::InvalidPath => fl!(LOADER, "err-invalid-path"),
        ErrorKind::NoSpaceLeft => fl!(LOADER, "err-no-space"),
        ErrorKind::Cancelled => fl!(LOADER, "err-cancelled"),
        ErrorKind::ReadFailed => fl!(LOADER, "err-read"),
        ErrorKind::WriteFailed => fl!(LOADER, "err-write"),
        ErrorKind::NotInitialized => fl!(LOADER, "err-not-initialized"),
        ErrorKind::AbiMismatch => fl!(LOADER, "err-abi-mismatch"),
        ErrorKind::InvalidEntry => fl!(LOADER, "err-invalid-entry"),
        ErrorKind::NotAPartition => fl!(LOADER, "err-not-a-partition"),
        ErrorKind::WrongPartitionType => fl!(LOADER, "err-wrong-partition-type"),
        ErrorKind::UnsupportedFilesystem => fl!(LOADER, "err-unsupported-filesystem"),
        ErrorKind::ToolNotInstalled => fl!(LOADER, "err-tool-not-installed"),
        ErrorKind::ToolFailed => fl!(LOADER, "err-tool-failed"),
        ErrorKind::RollbackFailed => fl!(LOADER, "err-rollback-failed"),
    }
}

#[repr(transparent)]
#[derive(Debug)]
pub struct LibError(pub AbiError);

impl Display for LibError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FmtResult {
        let stage = StageName::new(self.0.domain, self.0.state);

        match &self.0.subject {
            Some(subject) => write!(formatter, "{stage} ({subject}): {}", error_kind_message(self.0.kind)),
            None => write!(formatter, "{stage}: {}", error_kind_message(self.0.kind)),
        }
    }
}

impl Error for LibError {}
