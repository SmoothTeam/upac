// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 JustPav
//
// SPDX-License-Identifier: LGPL-3.0-or-later

use upac_abi::error::{CommandState, ErrorDomain};
use upac_macro::FromStageIndex;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, FromStageIndex)]
pub enum InstallStateId {
    PreHooks = 0,
    Preparation = 1,
    Transaction = 2,
    Merge = 3,
    Checkout = 4,
    Swap = 5,
    PostHooks = 6,
    Done = 7,
    Setup = 8,
}

impl CommandState for InstallStateId {
    const DOMAIN: ErrorDomain = ErrorDomain::Install;
    const VALIDATION: Self = InstallStateId::Setup;

    fn as_u32(self) -> u32 {
        self as u32
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, FromStageIndex)]
pub enum UninstallStateId {
    PreHooks = 0,
    Preparation = 1,
    Build = 2,
    Commit = 3,
    ConfigMerge = 4,
    PrepareBoot = 5,
    BootOption = 6,
    PostHooks = 7,
    Done = 8,
    Setup = 9,
}

impl CommandState for UninstallStateId {
    const DOMAIN: ErrorDomain = ErrorDomain::Uninstall;
    const VALIDATION: Self = UninstallStateId::Setup;

    fn as_u32(self) -> u32 {
        self as u32
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, FromStageIndex)]
pub enum RollbackStateId {
    PreHooks = 0,
    Merge = 1,
    Checkout = 2,
    Swap = 3,
    PostHooks = 4,
    Done = 5,
    Setup = 6,
}

impl CommandState for RollbackStateId {
    const DOMAIN: ErrorDomain = ErrorDomain::Rollback;
    const VALIDATION: Self = RollbackStateId::Setup;

    fn as_u32(self) -> u32 {
        self as u32
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, FromStageIndex)]
pub enum FilesStateId {
    PreHooks = 0,
    Transaction = 1,
    Checkout = 2,
    Swap = 3,
    PostHooks = 4,
    Done = 5,
    Setup = 6,
}

impl CommandState for FilesStateId {
    const DOMAIN: ErrorDomain = ErrorDomain::Files;
    const VALIDATION: Self = FilesStateId::Setup;

    fn as_u32(self) -> u32 {
        self as u32
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, FromStageIndex)]
pub enum ListPackagesStateId {
    Fetching = 0,
    Done = 1,
    Setup = 2,
}

impl CommandState for ListPackagesStateId {
    const DOMAIN: ErrorDomain = ErrorDomain::ListPackages;
    const VALIDATION: Self = ListPackagesStateId::Setup;

    fn as_u32(self) -> u32 {
        self as u32
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, FromStageIndex)]
pub enum ListCommitStateId {
    Fetching = 0,
    Done = 1,
    Setup = 2,
}

impl CommandState for ListCommitStateId {
    const DOMAIN: ErrorDomain = ErrorDomain::ListCommit;
    const VALIDATION: Self = ListCommitStateId::Setup;

    fn as_u32(self) -> u32 {
        self as u32
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, FromStageIndex)]
pub enum ListPrefixStateId {
    Fetching = 0,
    Done = 1,
    Setup = 2,
}

impl CommandState for ListPrefixStateId {
    const DOMAIN: ErrorDomain = ErrorDomain::ListPrefix;
    const VALIDATION: Self = ListPrefixStateId::Setup;

    fn as_u32(self) -> u32 {
        self as u32
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, FromStageIndex)]
pub enum ListHistoryStateId {
    Fetching = 0,
    Done = 1,
    Setup = 2,
}

impl CommandState for ListHistoryStateId {
    const DOMAIN: ErrorDomain = ErrorDomain::ListHistory;
    const VALIDATION: Self = ListHistoryStateId::Setup;

    fn as_u32(self) -> u32 {
        self as u32
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, FromStageIndex)]
pub enum DiffPrefixStateId {
    Preparing = 0,
    Comparing = 1,
    Done = 2,
    Setup = 3,
}

impl CommandState for DiffPrefixStateId {
    const DOMAIN: ErrorDomain = ErrorDomain::DiffPrefix;
    const VALIDATION: Self = DiffPrefixStateId::Setup;

    fn as_u32(self) -> u32 {
        self as u32
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, FromStageIndex)]
pub enum DiffConfigStateId {
    Preparing = 0,
    Comparing = 1,
    Done = 2,
    Setup = 3,
}

impl CommandState for DiffConfigStateId {
    const DOMAIN: ErrorDomain = ErrorDomain::DiffConfig;
    const VALIDATION: Self = DiffConfigStateId::Setup;

    fn as_u32(self) -> u32 {
        self as u32
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, FromStageIndex)]
pub enum DiffPackagesStateId {
    Preparing = 0,
    Comparing = 1,
    Done = 2,
    Setup = 3,
}

impl CommandState for DiffPackagesStateId {
    const DOMAIN: ErrorDomain = ErrorDomain::DiffPackages;
    const VALIDATION: Self = DiffPackagesStateId::Setup;

    fn as_u32(self) -> u32 {
        self as u32
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, FromStageIndex)]
pub enum DiffStateId {
    Preparing = 0,
    Comparing = 1,
    Done = 2,
    Setup = 3,
}

impl CommandState for DiffStateId {
    const DOMAIN: ErrorDomain = ErrorDomain::Diff;
    const VALIDATION: Self = DiffStateId::Setup;

    fn as_u32(self) -> u32 {
        self as u32
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, FromStageIndex)]
pub enum UpdateStateId {
    PreHooks = 0,
    Preparation = 1,
    Transaction = 2,
    Merge = 3,
    Checkout = 4,
    Swap = 5,
    PostHooks = 6,
    Done = 7,
    Setup = 8,
}

impl CommandState for UpdateStateId {
    const DOMAIN: ErrorDomain = ErrorDomain::Update;
    const VALIDATION: Self = UpdateStateId::Setup;

    fn as_u32(self) -> u32 {
        self as u32
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, FromStageIndex)]
pub enum SearchMetaStateId {
    Searching = 0,
    Done = 1,
    Setup = 2,
}

impl CommandState for SearchMetaStateId {
    const DOMAIN: ErrorDomain = ErrorDomain::SearchMeta;
    const VALIDATION: Self = SearchMetaStateId::Setup;

    fn as_u32(self) -> u32 {
        self as u32
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, FromStageIndex)]
pub enum SearchFilesStateId {
    Searching = 0,
    Done = 1,
    Setup = 2,
}

impl CommandState for SearchFilesStateId {
    const DOMAIN: ErrorDomain = ErrorDomain::SearchFiles;
    const VALIDATION: Self = SearchFilesStateId::Setup;

    fn as_u32(self) -> u32 {
        self as u32
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, FromStageIndex)]
pub enum CommitStateId {
    PreHooks = 0,
    Transaction = 1,
    PostHooks = 2,
    Done = 3,
    Setup = 4,
}

impl CommandState for CommitStateId {
    const DOMAIN: ErrorDomain = ErrorDomain::Commit;
    const VALIDATION: Self = CommitStateId::Setup;

    fn as_u32(self) -> u32 {
        self as u32
    }
}
