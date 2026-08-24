// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: GPL-3.0-only

use std::error::Error;
use std::fmt::{Display, Formatter, Result as FmtResult};

use upac_abi::error::{CError, ErrorDomain, ErrorKind};

use upac_types::states::{
    CommitStateId, DiffConfigStateId, DiffPackagesStateId, DiffPrefixStateId, DiffStateId, FilesStateId, GcStateId,
    InstallStateId, ListConfigStateId, ListHistoryStateId, ListPackagesStateId, ListPrefixStateId, MimeStateId,
    PinStateId, RollbackStateId, SearchFilesStateId, SearchInMetaStateId, SearchInPackageFilesStateId,
    SearchMetaStateId, UninstallStateId, UpdateStateId,
};

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
            gettextrs::gettext("abi_version_mismatch"),
            self.got,
            self.expected
        )
    }
}

impl Error for AbiMismatch {}

pub(crate) struct StageName {
    domain: ErrorDomain,
    state: u32,
}

impl StageName {
    pub(crate) fn new(domain: ErrorDomain, state: u32) -> Self {
        StageName { domain, state }
    }
}

impl From<&CError> for StageName {
    fn from(error: &CError) -> Self {
        StageName::new(error.domain, error.state)
    }
}

impl Display for StageName {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FmtResult {
        let state = self.state as usize;

        let key = match self.domain {
            ErrorDomain::Uninstall => UninstallStateId::from_stage_index(state).stage_key(),
            ErrorDomain::Install => InstallStateId::from_stage_index(state).stage_key(),
            ErrorDomain::Rollback => RollbackStateId::from_stage_index(state).stage_key(),
            ErrorDomain::Commit => CommitStateId::from_stage_index(state).stage_key(),
            ErrorDomain::Files => FilesStateId::from_stage_index(state).stage_key(),
            ErrorDomain::Update => UpdateStateId::from_stage_index(state).stage_key(),
            ErrorDomain::Gc => GcStateId::from_stage_index(state).stage_key(),
            ErrorDomain::Pin => PinStateId::from_stage_index(state).stage_key(),
            ErrorDomain::Mime => MimeStateId::from_stage_index(state).stage_key(),
            ErrorDomain::ListPackages => ListPackagesStateId::from_stage_index(state).stage_key(),
            ErrorDomain::ListConfig => ListConfigStateId::from_stage_index(state).stage_key(),
            ErrorDomain::ListPrefix => ListPrefixStateId::from_stage_index(state).stage_key(),
            ErrorDomain::ListHistory => ListHistoryStateId::from_stage_index(state).stage_key(),
            ErrorDomain::DiffPrefix => DiffPrefixStateId::from_stage_index(state).stage_key(),
            ErrorDomain::DiffConfig => DiffConfigStateId::from_stage_index(state).stage_key(),
            ErrorDomain::DiffPackages => DiffPackagesStateId::from_stage_index(state).stage_key(),
            ErrorDomain::Diff => DiffStateId::from_stage_index(state).stage_key(),
            ErrorDomain::SearchMeta => SearchMetaStateId::from_stage_index(state).stage_key(),
            ErrorDomain::SearchFiles => SearchFilesStateId::from_stage_index(state).stage_key(),
            ErrorDomain::SearchInMeta => SearchInMetaStateId::from_stage_index(state).stage_key(),
            ErrorDomain::SearchInPackageFiles => SearchInPackageFilesStateId::from_stage_index(state).stage_key(),
        };

        write!(formatter, "{}", gettextrs::gettext(key))
    }
}

#[derive(Debug)]
pub struct LibError {
    pub error: CError,
}

impl Display for LibError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FmtResult {
        let key = match self.error.error {
            ErrorKind::Unexpected => "err_unexpected",
            ErrorKind::OutOfMemory => "err_oom",
            ErrorKind::NotFound => "err_not_found",
            ErrorKind::AlreadyExists => "err_already_exists",
            ErrorKind::PermissionDenied => "err_permission_denied",
            ErrorKind::InvalidPath => "err_invalid_path",
            ErrorKind::NoSpaceLeft => "err_no_space",
            ErrorKind::Cancelled => "err_cancelled",
            ErrorKind::ReadFailed => "err_read",
            ErrorKind::WriteFailed => "err_write",
            ErrorKind::NotInitialized => "err_not_initialized",
            ErrorKind::AbiMismatch => "err_abi_mismatch",
            ErrorKind::InvalidEntry => "err_invalid_entry",
        };
        write!(
            formatter,
            "{} ({:?}: {})",
            gettextrs::gettext(key),
            self.error.domain,
            StageName::from(&self.error)
        )
    }
}

impl Error for LibError {}
