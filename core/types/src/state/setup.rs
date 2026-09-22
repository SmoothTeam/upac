// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_macro::{FromStageIndex, StageKey};

use super::impl_command_state;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, FromStageIndex, StageKey)]
pub enum SetupStateId {
    PrepareSource = 0,
    EnumeratePackages = 1,
    UnpackPackage = 2,
    ImportPackage = 3,
    ImportSystem = 4,
    Kernel = 5,
    EmbedDatabase = 6,
    WriteDeployRecord = 7,
    StageBoot = 8,
    Setup = 9,
}

impl_command_state!(SetupStateId, Setup);
