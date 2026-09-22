// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_macro::{FromStageIndex, StageKey};

use super::impl_command_state;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, FromStageIndex, StageKey)]
pub enum InstallStateId {
    PreHooks = 0,
    Fetching = 1,
    Preparation = 2,
    OpenTransaction = 3,
    ImportPackage = 4,
    CommitTransaction = 5,
    Merge = 6,
    Checkout = 7,
    Swap = 8,
    PostHooks = 9,
    Done = 10,
    Setup = 11,
}

impl_command_state!(InstallStateId, Install);

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, FromStageIndex, StageKey)]
pub enum UninstallStateId {
    PreHooks = 0,
    Preparation = 1,
    OpenTransaction = 2,
    RemovePackage = 3,
    CommitTransaction = 4,
    Merge = 5,
    Checkout = 6,
    Swap = 7,
    PostHooks = 8,
    Done = 9,
    Setup = 10,
}

impl_command_state!(UninstallStateId, Uninstall);

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, FromStageIndex, StageKey)]
pub enum UpdateStateId {
    PreHooks = 0,
    Fetching = 1,
    Preparation = 2,
    OpenTransaction = 3,
    ImportPackage = 4,
    CommitTransaction = 5,
    Merge = 6,
    Checkout = 7,
    Swap = 8,
    PostHooks = 9,
    Done = 10,
    Setup = 11,
}

impl_command_state!(UpdateStateId, Update);

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, FromStageIndex, StageKey)]
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, FromStageIndex, StageKey)]
pub enum FilesStateId {
    PreHooks = 0,
    OpenTransaction = 1,
    ApplyFile = 2,
    CommitTransaction = 3,
    Checkout = 4,
    Swap = 5,
    PostHooks = 6,
    Done = 7,
    Setup = 8,
}

impl_command_state!(FilesStateId, Files);

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, FromStageIndex, StageKey)]
pub enum CommitStateId {
    PreHooks = 0,
    Transaction = 1,
    PostHooks = 2,
    Done = 3,
    Setup = 4,
}

impl_command_state!(CommitStateId, Commit);

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, FromStageIndex, StageKey)]
pub enum GcStateId {
    Pruning = 0,
    CollectRoots = 1,
    Cleaning = 2,
    Done = 3,
    Setup = 4,
}

impl_command_state!(GcStateId, Gc);

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, FromStageIndex, StageKey)]
pub enum PinStateId {
    SetPinned = 0,
    Done = 1,
    Setup = 2,
}

impl_command_state!(PinStateId, Pin);

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, FromStageIndex, StageKey)]
pub enum MimeStateId {
    Preparing = 0,
    Rendering = 1,
    Writing = 2,
    Done = 3,
    Setup = 4,
}

impl_command_state!(MimeStateId, Mime);
