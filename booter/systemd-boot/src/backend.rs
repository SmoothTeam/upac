// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::fs::File;
use std::io::ErrorKind as IoErrorKind;
use std::iter::once;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::process::Command;

use efivar::efi::{Variable, VariableFlags};
use efivar::{Error as EfivarError, VarManager, system};

use rustix::fs::{IFlags, ioctl_getflags, ioctl_setflags};

use uuid::Uuid;

use upac_booter_kit::BootBackend;

use upac_types::booter::{BootError, BootResourceKind};

use super::layout::efivarfs::PATH as EFIVARFS_PATH;
use super::layout::install::{COMMAND, ESP_PATH_FLAG, TOOL};
use super::layout::loader::{ENTRY_DEFAULT, ENTRY_ONE_SHOT, VENDOR_GUID};

pub struct SystemdBoot {
    manager: Box<dyn VarManager>,
}

impl BootBackend for SystemdBoot {
    type Error = BootError;

    fn new() -> Result<Self, BootError> {
        let manager = catch_unwind(AssertUnwindSafe(system)).map_err(|_| BootError::EfiUnavailable)?;

        Ok(Self { manager })
    }

    fn boot_resource_kind() -> BootResourceKind {
        BootResourceKind::Bls
    }

    fn set_one_shot(&mut self, entry_name: &str) -> Result<(), BootError> {
        self.write_loader_variable(ENTRY_ONE_SHOT, entry_name)
    }

    fn confirm_boot(&mut self, entry_name: &str, esp_mount_point: &str) -> Result<(), BootError> {
        let _ = esp_mount_point;

        self.write_loader_variable(ENTRY_DEFAULT, entry_name)
    }

    fn install(&mut self, esp_mount_point: &str, to_slot: &str, from_slot: &str) -> Result<(), BootError> {
        let _ = (to_slot, from_slot);

        match Command::new(TOOL)
            .arg(COMMAND)
            .arg(format!("{ESP_PATH_FLAG}{esp_mount_point}"))
            .status()
        {
            Ok(status) if status.success() => Ok(()),
            Ok(_) => Err(BootError::Unexpected),
            Err(error) if error.kind() == IoErrorKind::NotFound => Err(BootError::ToolNotInstalled),
            Err(error) => Err(error.into()),
        }
    }
}

impl SystemdBoot {
    fn write_loader_variable(&mut self, name: &str, entry_name: &str) -> Result<(), BootError> {
        fn clear_immutable(variable: &Variable) {
            let Ok(file) = File::open(format!("{EFIVARFS_PATH}/{variable}")) else {
                return;
            };

            let Ok(flags) = ioctl_getflags(&file) else {
                return;
            };

            if flags.contains(IFlags::IMMUTABLE) {
                let _ = ioctl_setflags(&file, flags.difference(IFlags::IMMUTABLE));
            }
        }

        let vendor_guid = Uuid::parse_str(VENDOR_GUID).map_err(|_| BootError::Unexpected)?;
        let variable = Variable::new_with_vendor(name, vendor_guid);

        clear_immutable(&variable);

        let value: Vec<u8> = entry_name
            .encode_utf16()
            .chain(once(0))
            .flat_map(u16::to_le_bytes)
            .collect();

        self.manager
            .write(&variable, VariableFlags::default(), &value)
            .map_err(|error| match error {
                EfivarError::PermissionDenied { .. } => BootError::PermissionDenied,
                _ => BootError::Unexpected,
            })
    }
}
