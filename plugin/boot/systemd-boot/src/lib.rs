// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::ffi::OsStr;
use std::fs::File;
use std::io::ErrorKind as IoErrorKind;
use std::iter::once;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::path::Path;
use std::process::Command;

use efivar::efi::{Variable, VariableFlags};
use efivar::{Error as EfivarError, VarManager, system};

use rustix::fs::{IFlags, ioctl_getflags, ioctl_setflags};

use uuid::Uuid;

use upac_abi::BOOT_ABI_VERSION;
use upac_abi::request::{CBootPluginEntryRequest, CBootPluginInstallRequest};
use upac_abi::response::CBootPluginInfo;

use upac_booter_kit::BootBackend;

use upac_types::booter::{BootError, BootResourceKind};
use upac_types::decoder::plugin_status;
use upac_types::request::{BootPluginEntryRequest, BootPluginInstallRequest};
use upac_types::response::BootPluginInfo;

use self::layout::efivarfs::PATH as EFIVARFS_PATH;
use self::layout::install::{COMMAND, ESP_PATH_FLAG, TOOL};
use self::layout::loader::{ENTRY_DEFAULT, ENTRY_ONE_SHOT, VENDOR_GUID};

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

        SystemdBoot::new()?.install(&request)
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

        SystemdBoot::new()?.set_one_shot(&request)
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

        SystemdBoot::new()?.set_default(&request)
    }

    if request.is_null() {
        return plugin_status(Err(BootError::InvalidRequest));
    }

    plugin_status(set_default_entry(unsafe { &*request }))
}

struct SystemdBoot {
    manager: Box<dyn VarManager>,
}

impl BootBackend for SystemdBoot {
    type Error = BootError;

    fn new() -> Result<Self, BootError> {
        let manager = catch_unwind(AssertUnwindSafe(system)).map_err(|_| BootError::EfiUnavailable)?;

        Ok(Self { manager })
    }

    fn install(&mut self, request: &BootPluginInstallRequest) -> Result<(), BootError> {
        match Command::new(TOOL)
            .arg(COMMAND)
            .arg(format!("{ESP_PATH_FLAG}{}", request.esp_mount_point))
            .status()
        {
            Ok(status) if status.success() => Ok(()),
            Ok(_) => Err(BootError::Unexpected),
            Err(error) if error.kind() == IoErrorKind::NotFound => Err(BootError::ToolNotInstalled),
            Err(error) => Err(error.into()),
        }
    }

    fn set_one_shot(&mut self, request: &BootPluginEntryRequest) -> Result<(), BootError> {
        self.write_loader_variable(ENTRY_ONE_SHOT, request)
    }

    fn set_default(&mut self, request: &BootPluginEntryRequest) -> Result<(), BootError> {
        self.write_loader_variable(ENTRY_DEFAULT, request)
    }
}

impl SystemdBoot {
    fn write_loader_variable(
        &mut self, variable_name: &str, request: &BootPluginEntryRequest,
    ) -> Result<(), BootError> {
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

        let entry_file_name = Path::new(&request.entry_path)
            .file_name()
            .and_then(OsStr::to_str)
            .ok_or(BootError::InvalidRequest)?;

        let vendor_guid = Uuid::parse_str(VENDOR_GUID).map_err(|_| BootError::Unexpected)?;
        let variable = Variable::new_with_vendor(variable_name, vendor_guid);

        clear_immutable(&variable);

        let value: Vec<u8> = entry_file_name
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
