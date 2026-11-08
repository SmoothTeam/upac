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

use self::layout::efivarfs::PATH as EFIVARFS_PATH;
use self::layout::install::{COMMAND, ESP_PATH_FLAG, TOOL};
use self::layout::loader::{ENTRY_DEFAULT, ENTRY_ONE_SHOT, VENDOR_GUID};
use self::layout::manifest::NAME;

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
    SystemdBoot::boot_resource_kind().into()
}

/// # Safety
/// `request`, if non-null, must point to a valid, initialized `CBootPluginSetOneShotRequest` for the
/// duration of the call.
#[cfg_attr(feature = "cdylib", unsafe(no_mangle))]
pub unsafe extern "C" fn set_one_shot(request: *const CBootPluginSetOneShotRequest) -> i32 {
    fn set_one_shot_entry(request: &CBootPluginSetOneShotRequest) -> Result<(), BootError> {
        let request = BootPluginSetOneShotRequest::try_from(request)?;

        SystemdBoot::new()?.set_one_shot(&request.entry_name)
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

        SystemdBoot::new()?.confirm_boot(&request.entry_name, &request.esp_mount_point)
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

        SystemdBoot::new()?.install(&request.esp_mount_point, &request.to_slot, &request.from_slot)
    }

    if request.is_null() {
        return plugin_status(Err(BootError::InvalidRequest));
    }

    plugin_status(install_booter(unsafe { &*request }))
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
