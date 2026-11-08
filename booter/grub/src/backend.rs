// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::fs::{create_dir_all, write};
use std::io::ErrorKind as IoErrorKind;
use std::path::Path;
use std::process::Command;

use upac_booter_kit::BootBackend;

use upac_types::booter::{BootError, BootResourceKind};

use super::layout::config::{CONTENTS, DIRECTORY, FILE};
use super::layout::install::{
    BOOT_DIRECTORY_FLAG, BOOTLOADER_ID, BOOTLOADER_ID_FLAG, EFI_DIRECTORY_FLAG, TARGET, TARGET_FLAG,
    TOOLS as INSTALL_TOOLS,
};
use super::layout::reboot::TOOLS as REBOOT_TOOLS;
use super::layout::set_default::TOOLS as SET_DEFAULT_TOOLS;

pub struct Grub;

impl BootBackend for Grub {
    type Error = BootError;

    fn new() -> Result<Self, BootError> {
        Ok(Grub)
    }

    fn boot_resource_kind() -> BootResourceKind {
        BootResourceKind::Bls
    }

    fn set_one_shot(&mut self, entry_name: &str) -> Result<(), BootError> {
        run_installed_tool(REBOOT_TOOLS, &[entry_name])
    }

    fn confirm_boot(&mut self, entry_name: &str, esp_mount_point: &str) -> Result<(), BootError> {
        let _ = esp_mount_point;

        run_installed_tool(SET_DEFAULT_TOOLS, &[entry_name])
    }

    fn install(&mut self, esp_mount_point: &str, to_slot: &str, from_slot: &str) -> Result<(), BootError> {
        let _ = (to_slot, from_slot);

        run_installed_tool(
            INSTALL_TOOLS,
            &[
                &format!("{TARGET_FLAG}{TARGET}"),
                &format!("{EFI_DIRECTORY_FLAG}{esp_mount_point}"),
                &format!("{BOOT_DIRECTORY_FLAG}{esp_mount_point}"),
                &format!("{BOOTLOADER_ID_FLAG}{BOOTLOADER_ID}"),
            ],
        )?;

        let config_directory = Path::new(esp_mount_point).join(DIRECTORY);
        create_dir_all(&config_directory)?;
        write(config_directory.join(FILE), CONTENTS)?;

        Ok(())
    }
}

fn run_installed_tool(tools: &[&str], arguments: &[&str]) -> Result<(), BootError> {
    for tool in tools {
        match Command::new(tool).args(arguments).status() {
            Ok(status) if status.success() => return Ok(()),
            Ok(_) => return Err(BootError::Unexpected),
            Err(error) if error.kind() == IoErrorKind::NotFound => continue,
            Err(error) => return Err(error.into()),
        }
    }

    Err(BootError::ToolNotInstalled)
}
