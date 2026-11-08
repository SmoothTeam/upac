// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::fs::{create_dir_all, write};
use std::io::ErrorKind as IoErrorKind;
use std::path::Path;
use std::process::Command;

use upac_abi::BOOT_ABI_VERSION;
use upac_abi::request::booter::{
    CBootPluginConfirmSuccessBootRequest, CBootPluginInstallRequest, CBootPluginSetOneShotRequest,
};

use upac_booter_kit::{BootBackend, BuiltinBooter};

use upac_types::booter::{BootError, BootResourceKind};
use upac_types::plugin::plugin_status;
use upac_types::request::booter::{
    BootPluginConfirmSuccessBootRequest, BootPluginInstallRequest, BootPluginSetOneShotRequest,
};

use self::layout::config::{CONTENTS, DIRECTORY, FILE};
use self::layout::install::{
    BOOT_DIRECTORY_FLAG, BOOTLOADER_ID, BOOTLOADER_ID_FLAG, EFI_DIRECTORY_FLAG, TARGET, TARGET_FLAG,
    TOOLS as INSTALL_TOOLS,
};
use self::layout::manifest::NAME;
use self::layout::reboot::TOOLS as REBOOT_TOOLS;
use self::layout::set_default::TOOLS as SET_DEFAULT_TOOLS;

mod layout {
    include!(concat!(env!("OUT_DIR"), "/layout.rs"));
}

pub const BOOTER: BuiltinBooter = BuiltinBooter {
    name: NAME,
    boot_resource_kind,
    set_one_shot,
    confirm_boot,
    install,
};

#[cfg_attr(feature = "cdylib", unsafe(no_mangle))]
pub extern "C" fn boot_abi_version() -> u32 {
    BOOT_ABI_VERSION
}

#[cfg_attr(feature = "cdylib", unsafe(no_mangle))]
pub extern "C" fn boot_resource_kind() -> u8 {
    Grub::boot_resource_kind().into()
}

/// # Safety
/// `request`, if non-null, must point to a valid, initialized `CBootPluginSetOneShotRequest` for the
/// duration of the call.
#[cfg_attr(feature = "cdylib", unsafe(no_mangle))]
pub unsafe extern "C" fn set_one_shot(request: *const CBootPluginSetOneShotRequest) -> i32 {
    fn set_one_shot_entry(request: &CBootPluginSetOneShotRequest) -> Result<(), BootError> {
        let request = BootPluginSetOneShotRequest::try_from(request)?;

        Grub::new()?.set_one_shot(&request.entry_name)
    }

    if request.is_null() {
        return plugin_status(Err(BootError::InvalidRequest));
    }

    plugin_status(set_one_shot_entry(unsafe { &*request }))
}

/// # Safety
/// `request`, if non-null, must point to a valid, initialized `CBootPluginConfirmSuccessBootRequest` for
/// the duration of the call.
#[cfg_attr(feature = "cdylib", unsafe(no_mangle))]
pub unsafe extern "C" fn confirm_boot(request: *const CBootPluginConfirmSuccessBootRequest) -> i32 {
    fn confirm_boot_entry(request: &CBootPluginConfirmSuccessBootRequest) -> Result<(), BootError> {
        let request = BootPluginConfirmSuccessBootRequest::try_from(request)?;

        Grub::new()?.confirm_boot(&request.entry_name, &request.esp_mount_point)
    }

    if request.is_null() {
        return plugin_status(Err(BootError::InvalidRequest));
    }

    plugin_status(confirm_boot_entry(unsafe { &*request }))
}

/// # Safety
/// `request`, if non-null, must point to a valid, initialized `CBootPluginInstallRequest` for the
/// duration of the call.
#[cfg_attr(feature = "cdylib", unsafe(no_mangle))]
pub unsafe extern "C" fn install(request: *const CBootPluginInstallRequest) -> i32 {
    fn install_booter(request: &CBootPluginInstallRequest) -> Result<(), BootError> {
        let request = BootPluginInstallRequest::try_from(request)?;

        Grub::new()?.install(&request.esp_mount_point, &request.to_slot, &request.from_slot)
    }

    if request.is_null() {
        return plugin_status(Err(BootError::InvalidRequest));
    }

    plugin_status(install_booter(unsafe { &*request }))
}

struct Grub;

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
