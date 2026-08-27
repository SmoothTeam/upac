// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::panic::{AssertUnwindSafe, catch_unwind};
use std::path::Path;
use std::str::FromStr;

use efivar::VarManager;
use efivar::efi::{Variable, VariableFlags};

use uuid::Uuid;

use upac_abi::boot::Booter;

use crate::boot::{BOOT_NEXT_VAR, EFI_SYSFS_PATH, LOADER_INFO_VAR, SD_BOOT_LOADER_GUID};
use crate::error::UkiError;
use crate::grub::{GRUBENV_FALLBACK, GRUBENV_PRIMARY};
use crate::refind::{PREVIOUS_BOOT_GUID, PREVIOUS_BOOT_VAR};

pub struct Uki {
    manager: Box<dyn VarManager>,
}

impl Booter for Uki {
    type Error = UkiError;

    fn new() -> Result<Self, UkiError> {
        Ok(Self {
            manager: catch_unwind(AssertUnwindSafe(efivar::system))?,
        })
    }

    fn probes() -> bool {
        if !Path::new(EFI_SYSFS_PATH).exists() {
            return false;
        }
        if Path::new(GRUBENV_PRIMARY).exists() || Path::new(GRUBENV_FALLBACK).exists() {
            return false;
        }

        let Ok(manager) = catch_unwind(AssertUnwindSafe(efivar::system)) else {
            return false;
        };

        !efi_variable_exists(manager.as_ref(), LOADER_INFO_VAR, SD_BOOT_LOADER_GUID)
            && !efi_variable_exists(manager.as_ref(), PREVIOUS_BOOT_VAR, PREVIOUS_BOOT_GUID)
    }

    fn set_one_shot(&mut self, entry_name: &str) -> Result<(), UkiError> {
        let id = self.find_boot_id(entry_name)?;

        self.manager.write(
            &Variable::new(BOOT_NEXT_VAR),
            VariableFlags::default(),
            &id.to_le_bytes(),
        )?;

        Ok(())
    }

    fn confirm_boot(&mut self, entry_name: &str) -> Result<(), UkiError> {
        let id = self.find_boot_id(entry_name)?;

        let mut order = self.manager.get_boot_order()?;
        order.retain(|&existing| existing != id);
        order.insert(0, id);

        self.manager.set_boot_order(order)?;

        Ok(())
    }
}

impl Uki {
    fn find_boot_id(&self, slot_filename: &str) -> Result<u16, UkiError> {
        for (entry, _var) in self.manager.get_boot_entries()? {
            let entry = entry?;
            let matches = entry.entry.file_path_list.as_ref().is_some_and(|list| {
                list.file_path
                    .path
                    .to_lowercase()
                    .ends_with(&slot_filename.to_lowercase())
            });

            if matches {
                return Ok(entry.id);
            }
        }

        Err(UkiError::EntryNotFound)
    }
}

fn efi_variable_exists(manager: &dyn VarManager, name: &str, guid: &str) -> bool {
    let Ok(guid) = Uuid::from_str(guid) else {
        return false;
    };

    manager.exists(&Variable::new_with_vendor(name, guid)).unwrap_or(false)
}
