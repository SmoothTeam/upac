// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 JustPav
//
// SPDX-License-Identifier: LGPL-3.0-or-later

use std::str::FromStr;

use efivar::efi::{Variable, VariableFlags};

use uuid::Uuid;

use crate::boot::OneShotReboot;
use crate::boot::error::BootError;
use crate::layout::boot::{LOADER_ENTRY_DEFAULT_VAR, LOADER_ENTRY_ONE_SHOT_VAR, SD_BOOT_LOADER_GUID};

pub struct Bls;

impl OneShotReboot for Bls {
    fn set_one_shot(&self, entry_name: &str) -> Result<(), BootError> {
        let mut manager = efivar::system();
        let variable = Variable::new_with_vendor(LOADER_ENTRY_ONE_SHOT_VAR, Self::sd_boot_loader_guid()?);

        manager.write(
            &variable,
            VariableFlags::default(),
            &Self::encode_utf16_null(entry_name),
        )?;

        Ok(())
    }

    fn confirm_boot(&self, entry_name: &str) -> Result<(), BootError> {
        let mut manager = efivar::system();
        let variable = Variable::new_with_vendor(LOADER_ENTRY_DEFAULT_VAR, Self::sd_boot_loader_guid()?);

        manager.write(
            &variable,
            VariableFlags::default(),
            &Self::encode_utf16_null(entry_name),
        )?;

        Ok(())
    }
}

impl Bls {
    fn sd_boot_loader_guid() -> Result<Uuid, BootError> {
        Uuid::from_str(SD_BOOT_LOADER_GUID).map_err(|_| BootError::Unexpected)
    }

    fn encode_utf16_null(value: &str) -> Vec<u8> {
        let mut bytes: Vec<u8> = value.encode_utf16().flat_map(u16::to_le_bytes).collect();
        bytes.extend_from_slice(&[0x00, 0x00]);

        bytes
    }
}
