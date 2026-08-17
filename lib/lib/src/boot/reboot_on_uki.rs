// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 JustPav
//
// SPDX-License-Identifier: LGPL-3.0-or-later

use std::panic::{AssertUnwindSafe, catch_unwind};

use efivar::VarManager;
use efivar::efi::{Variable, VariableFlags};

use crate::boot::OneShotReboot;
use crate::boot::error::BootError;
use crate::layout::boot::BOOT_NEXT_VAR;

pub struct Uki {
    manager: Box<dyn VarManager>,
}

impl OneShotReboot for Uki {
    fn set_one_shot(&mut self, entry_name: &str) -> Result<(), BootError> {
        let id = Self::find_boot_id(self.manager.as_ref(), entry_name)?;

        self.manager.write(
            &Variable::new(BOOT_NEXT_VAR),
            VariableFlags::default(),
            &id.to_le_bytes(),
        )?;

        Ok(())
    }

    fn confirm_boot(&mut self, entry_name: &str) -> Result<(), BootError> {
        let id = Self::find_boot_id(self.manager.as_ref(), entry_name)?;

        let mut order = self.manager.get_boot_order()?;
        order.retain(|&existing| existing != id);
        order.insert(0, id);
        self.manager.set_boot_order(order)?;

        Ok(())
    }
}

impl Uki {
    pub fn new() -> Result<Self, BootError> {
        Ok(Self {
            manager: catch_unwind(AssertUnwindSafe(efivar::system))?,
        })
    }

    fn find_boot_id(manager: &dyn VarManager, slot_filename: &str) -> Result<u16, BootError> {
        for (entry, _var) in manager.get_boot_entries()? {
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

        Err(BootError::BootEntryNotFound)
    }
}
