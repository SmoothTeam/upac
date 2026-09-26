// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

pub mod error;
pub mod hook;
pub mod memory;
pub mod package;
pub mod plugin;
pub mod request;
pub mod response;
pub mod types;

pub const LIB_ABI_VERSION: u32 = 2;
pub const BOOT_ABI_VERSION: u32 = 3;
pub const DECODER_ABI_VERSION: u32 = 2;
pub const SETUP_ABI_VERSION: u32 = 2;

pub const CONSTRAINT_LESS: u8 = 0b001;
pub const CONSTRAINT_EQUAL: u8 = 0b010;
pub const CONSTRAINT_GREATER: u8 = 0b100;
pub const CONSTRAINT_ANY: u8 = CONSTRAINT_LESS | CONSTRAINT_EQUAL | CONSTRAINT_GREATER;
