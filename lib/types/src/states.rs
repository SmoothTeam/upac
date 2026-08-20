// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later

use upac_abi::error::{CommandState, ErrorDomain};

use upac_macro::FromStageIndex;

macro_rules! impl_command_state {
    ($name:ident, $domain:ident) => {
        impl CommandState for $name {
            const DOMAIN: ErrorDomain = ErrorDomain::$domain;
            const VALIDATION: Self = $name::Setup;

            fn as_u32(self) -> u32 {
                self as u32
            }
        }
    };
}

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

impl_command_state!(InstallStateId, Install);

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

impl_command_state!(UninstallStateId, Uninstall);

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

impl_command_state!(RollbackStateId, Rollback);

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

impl_command_state!(FilesStateId, Files);

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, FromStageIndex)]
pub enum GcStateId {
    Cleaning = 0,
    Done = 1,
    Setup = 2,
}

impl_command_state!(GcStateId, Gc);

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, FromStageIndex)]
pub enum MimeStateId {
    Preparing = 0,
    Rendering = 1,
    Writing = 2,
    Done = 3,
    Setup = 4,
}

impl_command_state!(MimeStateId, Mime);

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, FromStageIndex)]
pub enum ListPackagesStateId {
    Fetching = 0,
    Done = 1,
    Setup = 2,
}

impl_command_state!(ListPackagesStateId, ListPackages);

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, FromStageIndex)]
pub enum ListConfigStateId {
    Fetching = 0,
    Done = 1,
    Setup = 2,
}

impl_command_state!(ListConfigStateId, ListConfig);

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, FromStageIndex)]
pub enum ListPrefixStateId {
    Fetching = 0,
    Done = 1,
    Setup = 2,
}

impl_command_state!(ListPrefixStateId, ListPrefix);

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, FromStageIndex)]
pub enum ListHistoryStateId {
    Fetching = 0,
    Done = 1,
    Setup = 2,
}

impl_command_state!(ListHistoryStateId, ListHistory);

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, FromStageIndex)]
pub enum DiffPrefixStateId {
    Preparing = 0,
    Comparing = 1,
    Done = 2,
    Setup = 3,
}

impl_command_state!(DiffPrefixStateId, DiffPrefix);

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, FromStageIndex)]
pub enum DiffConfigStateId {
    Preparing = 0,
    Comparing = 1,
    Done = 2,
    Setup = 3,
}

impl_command_state!(DiffConfigStateId, DiffConfig);

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, FromStageIndex)]
pub enum DiffPackagesStateId {
    Preparing = 0,
    Comparing = 1,
    Done = 2,
    Setup = 3,
}

impl_command_state!(DiffPackagesStateId, DiffPackages);

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, FromStageIndex)]
pub enum DiffStateId {
    Preparing = 0,
    Comparing = 1,
    Done = 2,
    Setup = 3,
}

impl_command_state!(DiffStateId, Diff);

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

impl_command_state!(UpdateStateId, Update);

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, FromStageIndex)]
pub enum SearchMetaStateId {
    Searching = 0,
    Done = 1,
    Setup = 2,
}

impl_command_state!(SearchMetaStateId, SearchMeta);

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, FromStageIndex)]
pub enum SearchFilesStateId {
    Searching = 0,
    Done = 1,
    Setup = 2,
}

impl_command_state!(SearchFilesStateId, SearchFiles);

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, FromStageIndex)]
pub enum SearchInMetaStateId {
    Searching = 0,
    Done = 1,
    Setup = 2,
}

impl_command_state!(SearchInMetaStateId, SearchInMeta);

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, FromStageIndex)]
pub enum SearchInPackageFilesStateId {
    Searching = 0,
    Done = 1,
    Setup = 2,
}

impl_command_state!(SearchInPackageFilesStateId, SearchInPackageFiles);

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, FromStageIndex)]
pub enum CommitStateId {
    PreHooks = 0,
    Transaction = 1,
    PostHooks = 2,
    Done = 3,
    Setup = 4,
}

impl_command_state!(CommitStateId, Commit);
