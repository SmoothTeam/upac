// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_abi::BOOT_ABI_VERSION;
use upac_abi::request::booter::{
    CBootPluginConfirmSuccessBootRequest, CBootPluginInstallRequest, CBootPluginSetOneShotRequest,
};

use upac_booter_kit::{BootBackend, BuiltinBooter};

use upac_types::booter::BootError;
use upac_types::plugin::plugin_status;
use upac_types::request::booter::{
    BootPluginConfirmSuccessBootRequest, BootPluginInstallRequest, BootPluginSetOneShotRequest,
};

use self::backend::SystemdBoot;
use self::layout::manifest::NAME;

mod backend;
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
