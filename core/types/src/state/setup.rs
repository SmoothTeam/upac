// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_macro::{FromStageIndex, StageKey};

use super::impl_command_state;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, FromStageIndex, StageKey)]
pub enum BootstrapImportStateId {
    Prepare = 0,
    Unpack = 1,
    Add = 2,
    System = 3,
    Commit = 4,
    Config = 5,
    Setup = 6,
}

impl_command_state!(BootstrapImportStateId, BootstrapImport);

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, FromStageIndex, StageKey)]
pub enum BootstrapKernelStateId {
    Export = 0,
    Generate = 1,
    Commit = 2,
    Setup = 3,
}

impl_command_state!(BootstrapKernelStateId, BootstrapKernel);

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, FromStageIndex, StageKey)]
pub enum BootstrapDeployStateId {
    Register = 0,
    Boot = 1,
    Setup = 2,
}

impl_command_state!(BootstrapDeployStateId, BootstrapDeploy);

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
