// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 JustPav
//
// SPDX-License-Identifier: LGPL-3.0-or-later

use self::error::ErrorKind;

pub mod boot;
pub mod decoder;
pub mod error;
pub mod hook;
pub mod memory;
pub mod package;
pub mod request;
pub mod response;
pub mod types;

pub const ABI_VERSION: u32 = 2;
pub const BOOT_ABI_VERSION: u32 = 1;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileDiffKind {
    Added = 0,
    Removed = 1,
    Modified = 2,
}

impl FileDiffKind {
    pub fn from_u8(version: u8) -> Result<FileDiffKind, ErrorKind> {
        match version {
            0 => Ok(FileDiffKind::Added),
            1 => Ok(FileDiffKind::Removed),
            2 => Ok(FileDiffKind::Modified),
            _ => Err(ErrorKind::InvalidEntry),
        }
    }
}

// A package's own metadata can be Added/Removed/Modified — or unchanged while
// one of its own files changed underneath it (e.g. a hand-edited is_user file),
// which FileDiffKind's three variants can't represent. Kept separate rather
// than adding a fourth variant to FileDiffKind, since every file-level
// consumer (DiffPrefixFileEntry/DiffConfigFileEntry/DiffUntrackedFileEntry) is
// already a complete, correct 3-way split — a package-only concept doesn't
// belong there.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackageDiffKind {
    Added = 0,
    Removed = 1,
    Modified = 2,
    FilesChanged = 3,
}

impl PackageDiffKind {
    pub fn from_u8(version: u8) -> Result<PackageDiffKind, ErrorKind> {
        match version {
            0 => Ok(PackageDiffKind::Added),
            1 => Ok(PackageDiffKind::Removed),
            2 => Ok(PackageDiffKind::Modified),
            3 => Ok(PackageDiffKind::FilesChanged),
            _ => Err(ErrorKind::InvalidEntry),
        }
    }
}

// Distinguishes which tree a DiffPrefixFileEntry/DiffUntrackedFileEntry came
// from when both /usr and /etc changes are folded into one list (the combined
// diff command). Standalone diff_prefix/diff_config don't need it to
// disambiguate (the command itself already implies the axis), but reuse the
// same entry types and set it to a fixed value.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiffFileSource {
    Prefix = 0,
    Config = 1,
}

impl DiffFileSource {
    pub fn from_u8(version: u8) -> Result<DiffFileSource, ErrorKind> {
        match version {
            0 => Ok(DiffFileSource::Prefix),
            1 => Ok(DiffFileSource::Config),
            _ => Err(ErrorKind::InvalidEntry),
        }
    }
}

// Explicit override for which one-shot boot mechanism a mutating command's Checkout/Swap (or
// Uninstall's PrepareBoot/BootOption) stage should use, instead of relying on that stage's own
// autodetection. Auto is the default (no CLI flag given).
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BootKind {
    Auto = 0,
    Uki = 1,
    Bls = 2,
}

impl BootKind {
    pub fn from_u8(version: u8) -> Result<BootKind, ErrorKind> {
        match version {
            0 => Ok(BootKind::Auto),
            1 => Ok(BootKind::Uki),
            2 => Ok(BootKind::Bls),
            _ => Err(ErrorKind::InvalidEntry),
        }
    }
}
