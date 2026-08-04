// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 JustPav
//
// SPDX-License-Identifier: LGPL-3.0-or-later

use self::error::ErrorKind;

pub mod decoder;
pub mod error;
pub mod hook;
pub mod memory;
pub mod package;
pub mod request;
pub mod response;
pub mod types;

pub const ABI_VERSION: u32 = 2;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiffKind {
    Added = 0,
    Removed = 1,
    Modified = 2,
}

impl DiffKind {
    pub fn from_u8(version: u8) -> Result<DiffKind, ErrorKind> {
        match version {
            0 => Ok(DiffKind::Added),
            1 => Ok(DiffKind::Removed),
            2 => Ok(DiffKind::Modified),
            _ => Err(ErrorKind::InvalidEntry),
        }
    }
}
