// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_abi::error::ErrorDomain;
use upac_abi::hook::{CProgressEvent, HookAck};

use super::error::DecodeError;
use super::package::DecodedPackageMeta;

pub trait CommandState: Copy {
    const DOMAIN: ErrorDomain;
    const VALIDATION: Self;

    fn as_u32(self) -> u32;
}

pub trait Booter: Sized {
    type Error;

    fn new() -> Result<Self, Self::Error>;
    fn set_one_shot(&mut self, entry_name: &str) -> Result<(), Self::Error>;
    fn confirm_boot(&mut self, entry_name: &str, esp_mount_point: &str) -> Result<(), Self::Error>;

    #[allow(
        clippy::too_many_arguments,
        reason = "each param is its own ESP/partition fact every Booter impl needs individually (mount point, \
                  partition number, LBA range, GUID, slot names) — grouping them into a request type here would \
                  just be indirection, not a real builder-shaped API"
    )]
    fn install(
        &mut self, esp_mount_point: &str, esp_partition_number: u32, esp_starting_lba: u64, esp_ending_lba: u64,
        esp_unique_partition_guid: [u8; 16], to_slot: &str, from_slot: &str,
    ) -> Result<(), Self::Error>;
}

pub trait DecodeMeta {
    fn decode(&self, sha256: [u8; 32]) -> Result<DecodedPackageMeta, DecodeError>;
}

pub trait MessageHook {
    fn send(&self, event: &CProgressEvent) -> HookAck;
}
