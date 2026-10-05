// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_macro::{FromStageIndex, StageKey};

use super::impl_command_state;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, FromStageIndex, StageKey)]
pub enum InstallStateId {
    Unpack = 0,
    PreHooks = 1,
    Open = 2,
    Import = 3,
    Commit = 4,
    Merge = 5,
    Deploy = 6,
    PostHooks = 7,
    Checkout = 8,
    Swap = 9,
    Retention = 10,
    Done = 11,
    Setup = 12,
}

impl_command_state!(InstallStateId, Install);

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, FromStageIndex, StageKey)]
pub enum UninstallStateId {
    Open = 0,
    Prepare = 1,
    PreHooks = 2,
    Remove = 3,
    Commit = 4,
    Merge = 5,
    Deploy = 6,
    PostHooks = 7,
    Checkout = 8,
    Swap = 9,
    Retention = 10,
    Done = 11,
    Setup = 12,
}

impl_command_state!(UninstallStateId, Uninstall);

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, FromStageIndex, StageKey)]
pub enum UpdateStateId {
    Unpack = 0,
    PreHooks = 1,
    Open = 2,
    Import = 3,
    Commit = 4,
    Merge = 5,
    Deploy = 6,
    PostHooks = 7,
    Checkout = 8,
    Swap = 9,
    Retention = 10,
    Done = 11,
    Setup = 12,
}

impl_command_state!(UpdateStateId, Update);

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, FromStageIndex, StageKey)]
pub enum RollbackStateId {
    Select = 0,
    Checkout = 1,
    Swap = 2,
    Done = 3,
    Setup = 4,
}

impl_command_state!(RollbackStateId, Rollback);

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, FromStageIndex, StageKey)]
pub enum AttachStateId {
    Open = 0,
    Resolve = 1,
    Apply = 2,
    Commit = 3,
    Merge = 4,
    Deploy = 5,
    Checkout = 6,
    Swap = 7,
    Retention = 8,
    Done = 9,
    Setup = 10,
}

impl_command_state!(AttachStateId, Attach);

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, FromStageIndex, StageKey)]
pub enum DetachStateId {
    Open = 0,
    Resolve = 1,
    Apply = 2,
    Commit = 3,
    Merge = 4,
    Deploy = 5,
    Checkout = 6,
    Swap = 7,
    Retention = 8,
    Done = 9,
    Setup = 10,
}

impl_command_state!(DetachStateId, Detach);

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, FromStageIndex, StageKey)]
pub enum CommitStateId {
    Transaction = 0,
    Done = 1,
    Setup = 2,
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
