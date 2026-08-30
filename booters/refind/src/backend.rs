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

use crate::error::RefindError;
use crate::refind::{PREVIOUS_BOOT_GUID, PREVIOUS_BOOT_VAR};

pub struct Refind {
    manager: Box<dyn VarManager>,
}

/// rEFInd has a single variable, `PreviousBoot`, for both one-shot selection and persisting the
/// confirmed choice — it writes the variable itself before every launch, so a successful boot
/// through the requested entry already re-records it as `PreviousBoot`. `set_one_shot` and
/// `confirm_boot` therefore perform the identical write; there is no separate persistent-default
/// variable to distinguish them, unlike systemd-boot's `LoaderEntryOneShot`/`LoaderEntryDefault`.
/// Only takes effect if `refind.conf`'s `default_selection` starts with `+` — outside this
/// plugin's control, see `booter.toml`.
impl Booter for Refind {
    type Error = RefindError;

    fn new() -> Result<Self, RefindError> {
        Ok(Self {
            manager: catch_unwind(AssertUnwindSafe(efivar::system))?,
        })
    }

    fn probes() -> bool {
        let Ok(manager) = catch_unwind(AssertUnwindSafe(efivar::system)) else {
            return false;
        };
        let Ok(guid) = Uuid::from_str(PREVIOUS_BOOT_GUID) else {
            return false;
        };

        manager
            .exists(&Variable::new_with_vendor(PREVIOUS_BOOT_VAR, guid))
            .unwrap_or(false)
    }

    fn set_one_shot(&mut self, entry_name: &str) -> Result<(), RefindError> {
        self.write_previous_boot(entry_name)
    }

    fn confirm_boot(&mut self, entry_name: &str) -> Result<(), RefindError> {
        self.write_previous_boot(entry_name)
    }
}

impl Refind {
    fn write_previous_boot(&mut self, entry_name: &str) -> Result<(), RefindError> {
        let guid = Uuid::from_str(PREVIOUS_BOOT_GUID)?;
        let variable = Variable::new_with_vendor(PREVIOUS_BOOT_VAR, guid);

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
