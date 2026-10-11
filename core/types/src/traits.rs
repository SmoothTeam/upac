// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use super::booter::BootResourceKind;
use super::decoder::DecodeError;
use super::error::ErrorDomain;
use super::package::DecodedPackageMeta;

pub trait CommandState: Copy {
    const DOMAIN: ErrorDomain;
    const VALIDATION: Self;

    fn as_u32(self) -> u32;

    fn from_stage_index(index: usize) -> Self;
}

pub trait Booter: Sized {
    type Error;

    fn new() -> Result<Self, Self::Error>;
    fn boot_resource_kind() -> BootResourceKind;
    fn set_one_shot(&mut self, entry_name: &str) -> Result<(), Self::Error>;
    fn confirm_boot(&mut self, entry_name: &str, esp_mount_point: &str) -> Result<(), Self::Error>;
    fn install(&mut self, esp_mount_point: &str, to_slot: &str, from_slot: &str) -> Result<(), Self::Error>;
}

pub trait DecodeMeta {
    fn decode(&self, sha256: [u8; 32]) -> Result<DecodedPackageMeta, DecodeError>;
}
