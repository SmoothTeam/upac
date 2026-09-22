// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_macro::{FromStageIndex, StageKey};

use super::impl_command_state;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, FromStageIndex, StageKey)]
pub enum SearchMetaStateId {
    Searching = 0,
    Done = 1,
    Setup = 2,
}

impl_command_state!(SearchMetaStateId, SearchMeta);

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, FromStageIndex, StageKey)]
pub enum SearchFilesStateId {
    Searching = 0,
    Done = 1,
    Setup = 2,
}

impl_command_state!(SearchFilesStateId, SearchFiles);

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, FromStageIndex, StageKey)]
pub enum SearchInMetaStateId {
    Searching = 0,
    Done = 1,
    Setup = 2,
}

impl_command_state!(SearchInMetaStateId, SearchInMeta);

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, FromStageIndex, StageKey)]
pub enum SearchInPackageFilesStateId {
    Searching = 0,
    Done = 1,
    Setup = 2,
}

impl_command_state!(SearchInPackageFilesStateId, SearchInPackageFiles);

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, FromStageIndex, StageKey)]
pub enum ListPackagesStateId {
    Fetching = 0,
    Done = 1,
    Setup = 2,
}

impl_command_state!(ListPackagesStateId, ListPackages);

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, FromStageIndex, StageKey)]
pub enum ListConfigStateId {
    Fetching = 0,
    Done = 1,
    Setup = 2,
}

impl_command_state!(ListConfigStateId, ListConfig);

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, FromStageIndex, StageKey)]
pub enum ListPrefixStateId {
    Fetching = 0,
    Done = 1,
    Setup = 2,
}

impl_command_state!(ListPrefixStateId, ListPrefix);

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, FromStageIndex, StageKey)]
pub enum ListHistoryStateId {
    Fetching = 0,
    Done = 1,
    Setup = 2,
}

impl_command_state!(ListHistoryStateId, ListHistory);

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, FromStageIndex, StageKey)]
pub enum DiffPrefixStateId {
    Preparing = 0,
    Comparing = 1,
    Done = 2,
    Setup = 3,
}

impl_command_state!(DiffPrefixStateId, DiffPrefix);

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, FromStageIndex, StageKey)]
pub enum DiffConfigStateId {
    Preparing = 0,
    Comparing = 1,
    Done = 2,
    Setup = 3,
}

impl_command_state!(DiffConfigStateId, DiffConfig);

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, FromStageIndex, StageKey)]
pub enum DiffPackagesStateId {
    Preparing = 0,
    Comparing = 1,
    Done = 2,
    Setup = 3,
}

impl_command_state!(DiffPackagesStateId, DiffPackages);

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, FromStageIndex, StageKey)]
pub enum DiffStateId {
    Preparing = 0,
    Comparing = 1,
    Done = 2,
    Setup = 3,
}

impl_command_state!(DiffStateId, Diff);
