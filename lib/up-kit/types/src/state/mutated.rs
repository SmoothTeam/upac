// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_macro::{FromStageIndex, StageKey};

use super::impl_command_state;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, FromStageIndex, StageKey)]
pub enum InstallStateId {
    Open = 0,
    Decode = 1,
    Import = 2,
    Hooks = 3,
    Check = 4,
    Commit = 5,
    Deploy = 6,
    Boot = 7,
    Switch = 8,
    Retention = 9,
    Setup = 10,
}

impl_command_state!(InstallStateId, Install);

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, FromStageIndex, StageKey)]
pub enum UpdateStateId {
    Open = 0,
    Decode = 1,
    Import = 2,
    Hooks = 3,
    Check = 4,
    Commit = 5,
    Deploy = 6,
    Boot = 7,
    Switch = 8,
    Retention = 9,
    Setup = 10,
}

impl_command_state!(UpdateStateId, Update);

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, FromStageIndex, StageKey)]
pub enum UninstallStateId {
    Open = 0,
    Remove = 1,
    Hooks = 2,
    Check = 3,
    Commit = 4,
    Deploy = 5,
    Boot = 6,
    Switch = 7,
    Retention = 8,
    Setup = 9,
}

impl_command_state!(UninstallStateId, Uninstall);

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, FromStageIndex, StageKey)]
pub enum AttachStateId {
    Open = 0,
    Apply = 1,
    Commit = 2,
    Deploy = 3,
    Boot = 4,
    Switch = 5,
    Retention = 6,
    Setup = 7,
}

impl_command_state!(AttachStateId, Attach);

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, FromStageIndex, StageKey)]
pub enum DetachStateId {
    Open = 0,
    Apply = 1,
    Commit = 2,
    Deploy = 3,
    Boot = 4,
    Switch = 5,
    Retention = 6,
    Setup = 7,
}

impl_command_state!(DetachStateId, Detach);

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, FromStageIndex, StageKey)]
pub enum RollbackStateId {
    Select = 0,
    Deploy = 1,
    Boot = 2,
    Switch = 3,
    Setup = 4,
}

impl_command_state!(RollbackStateId, Rollback);

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, FromStageIndex, StageKey)]
pub enum GcStateId {
    Pruning = 0,
    CollectRoots = 1,
    Cleaning = 2,
    Setup = 3,
}

impl_command_state!(GcStateId, Gc);

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, FromStageIndex, StageKey)]
pub enum PinStateId {
    SetPinned = 0,
    Setup = 1,
}

impl_command_state!(PinStateId, Pin);

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, FromStageIndex, StageKey)]
pub enum MimeStateId {
    Preparing = 0,
    Rendering = 1,
    Writing = 2,
    Setup = 3,
}

impl_command_state!(MimeStateId, Mime);
