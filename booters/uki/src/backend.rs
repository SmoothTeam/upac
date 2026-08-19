// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 JustPav
//
// SPDX-License-Identifier: LGPL-3.0-or-later

use std::panic::{AssertUnwindSafe, catch_unwind};
use std::path::Path;
use std::str::FromStr;

use efivar::VarManager;
use efivar::efi::{Variable, VariableFlags};

use uuid::Uuid;

use upac_abi::boot::Booter;

use crate::boot::{BOOT_NEXT_VAR, EFI_SYSFS_PATH, LOADER_INFO_VAR, SD_BOOT_LOADER_GUID};
use crate::error::UkiError;

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

        let Ok(manager) = catch_unwind(AssertUnwindSafe(efivar::system)) else {
            return false;
        };
        let Ok(guid) = Uuid::from_str(SD_BOOT_LOADER_GUID) else {
            return false;
        };

        !manager
            .exists(&Variable::new_with_vendor(LOADER_INFO_VAR, guid))
            .unwrap_or(false)
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
