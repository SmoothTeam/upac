// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::ffi::c_void;

use self::error::ErrorKind;
use self::hook::{CProgressEvent, HookAck};
use self::request::booter::{
    CBootPluginConfirmSuccsesBootRequest, CBootPluginInstallRequest, CBootPluginSetOneShotRequest,
};
use self::request::decoder::CDecodeRequest;
use self::response::decoder::CDecodeResponse;

pub mod error;
pub mod hook;
pub mod memory;
pub mod package;
pub mod request;
pub mod response;
pub mod types;

pub const LIB_ABI_VERSION: u32 = 3;
pub const BOOT_ABI_VERSION: u32 = 3;
pub const DECODER_ABI_VERSION: u32 = 2;
pub const SETUP_ABI_VERSION: u32 = 3;

pub const CONSTRAINT_LESS: u8 = 0b001;
pub const CONSTRAINT_EQUAL: u8 = 0b010;
pub const CONSTRAINT_GREATER: u8 = 0b100;
pub const CONSTRAINT_ANY: u8 = CONSTRAINT_LESS | CONSTRAINT_EQUAL | CONSTRAINT_GREATER;

pub type BootPluginAbiVersionFn = unsafe extern "C" fn() -> u32;

pub type DecodePluginAbiVersionFn = unsafe extern "C" fn() -> u32;

pub type HookMessageFn = unsafe extern "C" fn(event: *const CProgressEvent, ctx: *mut c_void) -> HookAck;

pub type SetOneShotFn =
    unsafe extern "C" fn(request: *const CBootPluginSetOneShotRequest, err_out: *mut ErrorKind) -> i32;

pub type ConfirmBootFn =
    unsafe extern "C" fn(request: *const CBootPluginConfirmSuccsesBootRequest, err_out: *mut ErrorKind) -> i32;

pub type InstallFn = unsafe extern "C" fn(request: *const CBootPluginInstallRequest, err_out: *mut ErrorKind) -> i32;

pub type BootResourceKindFn = unsafe extern "C" fn() -> BootResourceKind;

pub type DecodeFn = unsafe extern "C" fn(request: *const CDecodeRequest, response_out: *mut CDecodeResponse) -> i32;

pub type FreeDecodeResponseFn = unsafe extern "C" fn(response: *mut CDecodeResponse);

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BootResourceKind {
    Bls = 0,
    Uki = 1,
}

impl BootResourceKind {
    pub fn from_u8(version: u8) -> Result<BootResourceKind, ErrorKind> {
        match version {
            0 => Ok(BootResourceKind::Bls),
            1 => Ok(BootResourceKind::Uki),
            _ => Err(ErrorKind::InvalidEntry),
        }
    }
}

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

    pub fn as_str(&self) -> &'static str {
        match self {
            DiffFileSource::Prefix => "prefix",
            DiffFileSource::Config => "config",
        }
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FsKind {
    Ext4 = 0,
    Btrfs = 1,
    Xfs = 2,
    Vfat = 3,
}

impl FsKind {
    pub fn from_u8(version: u8) -> Result<FsKind, ErrorKind> {
        match version {
            0 => Ok(FsKind::Ext4),
            1 => Ok(FsKind::Btrfs),
            2 => Ok(FsKind::Xfs),
            3 => Ok(FsKind::Vfat),
            _ => Err(ErrorKind::InvalidEntry),
        }
    }
}

impl AsRef<str> for FsKind {
    fn as_ref(&self) -> &str {
        match self {
            FsKind::Ext4 => "ext4",
            FsKind::Btrfs => "btrfs",
            FsKind::Xfs => "xfs",
            FsKind::Vfat => "vfat",
        }
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PartitionKind {
    Esp = 0,
    Root = 1,
    Linux = 2,
}

impl PartitionKind {
    pub fn from_u8(version: u8) -> Result<PartitionKind, ErrorKind> {
        match version {
            0 => Ok(PartitionKind::Esp),
            1 => Ok(PartitionKind::Root),
            2 => Ok(PartitionKind::Linux),
            _ => Err(ErrorKind::InvalidEntry),
        }
    }
}

impl AsRef<str> for PartitionKind {
    fn as_ref(&self) -> &str {
        match self {
            PartitionKind::Esp => "esp",
            PartitionKind::Root => "root",
            PartitionKind::Linux => "linux",
        }
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InitramfsGenerator {
    Dracut = 0,
    Mkinitcpio = 1,
}

impl InitramfsGenerator {
    pub fn from_u8(version: u8) -> Result<InitramfsGenerator, ErrorKind> {
        match version {
            0 => Ok(InitramfsGenerator::Dracut),
            1 => Ok(InitramfsGenerator::Mkinitcpio),
            _ => Err(ErrorKind::InvalidEntry),
        }
    }
}

impl AsRef<str> for InitramfsGenerator {
    fn as_ref(&self) -> &str {
        match self {
            InitramfsGenerator::Dracut => "dracut",
            InitramfsGenerator::Mkinitcpio => "mkinitcpio",
        }
    }
}
