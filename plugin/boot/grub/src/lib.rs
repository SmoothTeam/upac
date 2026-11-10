// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::fs::{create_dir_all, write};
use std::io::ErrorKind as IoErrorKind;
use std::path::Path;
use std::process::Command;

use upac_abi::BOOT_ABI_VERSION;
use upac_abi::request::{CBootPluginEntryRequest, CBootPluginInstallRequest};
use upac_abi::response::CBootPluginInfo;

use upac_booter_kit::BootBackend;

use upac_types::booter::{BootError, BootResourceKind};
use upac_types::decoder::plugin_status;
use upac_types::request::{BootPluginEntryRequest, BootPluginInstallRequest};
use upac_types::response::BootPluginInfo;

use self::layout::config::{CONTENTS, DIRECTORY, FILE};
use self::layout::install::{
    BOOT_DIRECTORY_FLAG, BOOTLOADER_ID, BOOTLOADER_ID_FLAG, EFI_DIRECTORY_FLAG, TARGET, TARGET_FLAG,
    TOOLS as INSTALL_TOOLS,
};
use self::layout::reboot::TOOLS as REBOOT_TOOLS;
use self::layout::set_default::TOOLS as SET_DEFAULT_TOOLS;

mod layout {
    include!(concat!(env!("OUT_DIR"), "/layout.rs"));
}

#[unsafe(no_mangle)]
pub extern "C" fn boot_plugin_info() -> CBootPluginInfo {
    BootPluginInfo {
        abi_version: BOOT_ABI_VERSION,
        kind: BootResourceKind::Bls,
    }
    .into()
}

/// # Safety
/// `request`, if non-null, must point to a valid, initialized `CBootPluginInstallRequest` for the
/// duration of the call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn install(request: *const CBootPluginInstallRequest) -> i32 {
    fn install_booter(request: &CBootPluginInstallRequest) -> Result<(), BootError> {
        let request = BootPluginInstallRequest::try_from(request)?;

        Grub::new()?.install(&request)
    }

    if request.is_null() {
        return plugin_status(Err(BootError::InvalidRequest));
    }

    plugin_status(install_booter(unsafe { &*request }))
}

/// # Safety
/// `request`, if non-null, must point to a valid, initialized `CBootPluginEntryRequest` for the
/// duration of the call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn set_one_shot(request: *const CBootPluginEntryRequest) -> i32 {
    fn set_one_shot_entry(request: &CBootPluginEntryRequest) -> Result<(), BootError> {
        let request = BootPluginEntryRequest::try_from(request)?;

        Grub::new()?.set_one_shot(&request)
    }

    if request.is_null() {
        return plugin_status(Err(BootError::InvalidRequest));
    }

    plugin_status(set_one_shot_entry(unsafe { &*request }))
}

/// # Safety
/// `request`, if non-null, must point to a valid, initialized `CBootPluginEntryRequest` for the
/// duration of the call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn set_default(request: *const CBootPluginEntryRequest) -> i32 {
    fn set_default_entry(request: &CBootPluginEntryRequest) -> Result<(), BootError> {
        let request = BootPluginEntryRequest::try_from(request)?;

        Grub::new()?.set_default(&request)
    }

    if request.is_null() {
        return plugin_status(Err(BootError::InvalidRequest));
    }

    plugin_status(set_default_entry(unsafe { &*request }))
}

struct Grub;

impl BootBackend for Grub {
    type Error = BootError;

    fn new() -> Result<Self, BootError> {
        Ok(Grub)
    }

    fn install(&mut self, request: &BootPluginInstallRequest) -> Result<(), BootError> {
        let esp_mount_point = &request.esp_mount_point;

        for tool in INSTALL_TOOLS {
            match Command::new(tool)
                .args(&[
                    &format!("{TARGET_FLAG}{TARGET}"),
                    &format!("{EFI_DIRECTORY_FLAG}{esp_mount_point}"),
                    &format!("{BOOT_DIRECTORY_FLAG}{esp_mount_point}"),
                    &format!("{BOOTLOADER_ID_FLAG}{BOOTLOADER_ID}"),
                ])
                .status()
            {
                Ok(status) if status.success() => {
                    let config_directory = Path::new(esp_mount_point).join(DIRECTORY);
                    create_dir_all(&config_directory)?;
                    write(config_directory.join(FILE), CONTENTS)?;

                    return Ok(());
                }
                Ok(_) => return Err(BootError::Unexpected),
                Err(error) if error.kind() == IoErrorKind::NotFound => continue,
                Err(error) => return Err(error.into()),
            }
        }

        Err(BootError::ToolNotInstalled)
    }

    fn set_one_shot(&mut self, request: &BootPluginEntryRequest) -> Result<(), BootError> {
        for tool in REBOOT_TOOLS {
            match Command::new(tool)
                .args(&[
                    &format!("{BOOT_DIRECTORY_FLAG}{}", request.esp_mount_point),
                    &request.entry_id,
                ])
                .status()
            {
                Ok(status) if status.success() => return Ok(()),
                Ok(_) => return Err(BootError::Unexpected),
                Err(error) if error.kind() == IoErrorKind::NotFound => continue,
                Err(error) => return Err(error.into()),
            }
        }

        Err(BootError::ToolNotInstalled)
    }

    fn set_default(&mut self, request: &BootPluginEntryRequest) -> Result<(), BootError> {
        for tool in SET_DEFAULT_TOOLS {
            match Command::new(tool)
                .args(&[
                    &format!("{BOOT_DIRECTORY_FLAG}{}", request.esp_mount_point),
                    &request.entry_id,
                ])
                .status()
            {
                Ok(status) if status.success() => return Ok(()),
                Ok(_) => return Err(BootError::Unexpected),
                Err(error) if error.kind() == IoErrorKind::NotFound => continue,
                Err(error) => return Err(error.into()),
            }
        }

        Err(BootError::ToolNotInstalled)
    }
}
