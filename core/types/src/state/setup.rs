// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_macro::{FromStageIndex, StageKey};

use super::impl_command_state;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, FromStageIndex, StageKey)]
pub enum BootstrapStateId {
    Mount = 0,
    PrepareSource = 1,
    EnumeratePackages = 2,
    UnpackPackage = 3,
    ImportPackage = 4,
    ImportSystem = 5,
    Kernel = 6,
    EmbedDatabase = 7,
    WriteDeployRecord = 8,
    StageBoot = 9,
    Setup = 10,
}

impl_command_state!(BootstrapStateId, Bootstrap);

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, FromStageIndex, StageKey)]
pub enum PartitionTableStateId {
    Wipe = 0,
    WriteTable = 1,
    Setup = 2,
}

impl_command_state!(PartitionTableStateId, PartitionTable);

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, FromStageIndex, StageKey)]
pub enum PartitionAddStateId {
    InsertEntry = 0,
    Settle = 1,
    Setup = 2,
}

impl_command_state!(PartitionAddStateId, PartitionAdd);

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, FromStageIndex, StageKey)]
pub enum FormatStateId {
    Verify = 0,
    Wipe = 1,
    Mkfs = 2,
    Setup = 3,
}

impl_command_state!(FormatStateId, Format);
