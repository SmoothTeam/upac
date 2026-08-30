// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::panic::{AssertUnwindSafe, catch_unwind};
use std::str::FromStr;

use efivar::VarManager;
use efivar::efi::{Variable, VariableFlags};

use uuid::Uuid;

use upac_abi::boot::Booter;

use crate::boot::{LOADER_ENTRY_DEFAULT_VAR, LOADER_ENTRY_ONE_SHOT_VAR, LOADER_INFO_VAR, SD_BOOT_LOADER_GUID};
use crate::error::BlsError;

pub struct Bls {
    manager: Box<dyn VarManager>,
}

impl Booter for Bls {
    type Error = BlsError;

    fn new() -> Result<Self, BlsError> {
        Ok(Self {
            manager: catch_unwind(AssertUnwindSafe(efivar::system))?,
        })
    }

    fn probes() -> bool {
        let Ok(manager) = catch_unwind(AssertUnwindSafe(efivar::system)) else {
            return false;
        };
        let Ok(guid) = Uuid::from_str(SD_BOOT_LOADER_GUID) else {
            return false;
        };

        manager
            .exists(&Variable::new_with_vendor(LOADER_INFO_VAR, guid))
            .unwrap_or(false)
    }

    fn set_one_shot(&mut self, entry_name: &str) -> Result<(), BlsError> {
        self.write_loader_variable(LOADER_ENTRY_ONE_SHOT_VAR, entry_name)
    }

    fn confirm_boot(&mut self, entry_name: &str) -> Result<(), BlsError> {
        self.write_loader_variable(LOADER_ENTRY_DEFAULT_VAR, entry_name)
    }
}

impl Bls {
    fn write_loader_variable(&mut self, name: &str, entry_name: &str) -> Result<(), BlsError> {
        let guid = Uuid::from_str(SD_BOOT_LOADER_GUID)?;
        let variable = Variable::new_with_vendor(name, guid);

        self.manager
            .write(&variable, VariableFlags::default(), &encode_utf16_null(entry_name))?;

        Ok(())
    }
}

fn encode_utf16_null(value: &str) -> Vec<u8> {
    let mut bytes: Vec<u8> = value.encode_utf16().flat_map(u16::to_le_bytes).collect();
    bytes.extend_from_slice(&[0x00, 0x00]);

    bytes
}
