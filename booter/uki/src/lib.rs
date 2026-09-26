// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_abi::BOOT_ABI_VERSION;
use upac_abi::error::ErrorKind;
use upac_abi::request::booter::{
    CBootPluginConfirmSuccessBootRequest, CBootPluginInstallRequest, CBootPluginSetOneShotRequest,
};

use upac_types::request::booter::{
    BootPluginConfirmSuccessBootRequest, BootPluginInstallRequest, BootPluginSetOneShotRequest,
};
use upac_types::traits::Booter;

use self::backend::Uki;
use self::error::UkiError;

mod backend;
mod error;

include!(concat!(env!("OUT_DIR"), "/layout.rs"));

macro_rules! error_code {
    ($error:expr) => {
        u32::from(ErrorKind::from($error)) as i32
    };
}

/// # Safety
/// Touches no pointers — `unsafe extern "C"` only to match `upac_abi::boot::AbiVersionFn`.
#[cfg_attr(feature = "cdylib", unsafe(no_mangle))]
pub unsafe extern "C" fn boot_abi_version() -> u32 {
    BOOT_ABI_VERSION
}

/// # Safety
/// Touches no pointers — `unsafe extern "C"` only to match the ABI calling convention.
#[cfg_attr(feature = "cdylib", unsafe(no_mangle))]
pub unsafe extern "C" fn boot_resource_kind() -> u8 {
    Uki::boot_resource_kind().into()
}

/// # Safety
/// `request`, if non-null, must point to a valid, initialized `CBootPluginSetOneShotRequest` for the
/// duration of the call.
#[cfg_attr(feature = "cdylib", unsafe(no_mangle))]
pub unsafe extern "C" fn set_one_shot(request: *const CBootPluginSetOneShotRequest) -> i32 {
    if request.is_null() {
        return error_code!(UkiError::InvalidRequest);
    }

    let result = BootPluginSetOneShotRequest::try_from(unsafe { &*request })
        .map_err(UkiError::from)
        .and_then(|request| Uki::new().and_then(|mut uki| uki.set_one_shot(&request.entry_name)));

    match result {
        Ok(()) => 0,

        Err(error) => error_code!(error),
    }
}

/// # Safety
/// `request`, if non-null, must point to a valid, initialized `CBootPluginConfirmSuccessBootRequest` for the
/// duration of the call.
#[cfg_attr(feature = "cdylib", unsafe(no_mangle))]
pub unsafe extern "C" fn confirm_boot(request: *const CBootPluginConfirmSuccessBootRequest) -> i32 {
    if request.is_null() {
        return error_code!(UkiError::InvalidRequest);
    }

    let result = BootPluginConfirmSuccessBootRequest::try_from(unsafe { &*request })
        .map_err(UkiError::from)
        .and_then(|request| {
            Uki::new().and_then(|mut uki| uki.confirm_boot(&request.entry_name, &request.esp_mount_point))
        });

    match result {
        Ok(()) => 0,

        Err(error) => error_code!(error),
    }
}

/// # Safety
/// `request`, if non-null, must point to a valid, initialized `CBootPluginInstallRequest` for the
/// duration of the call.
#[cfg_attr(feature = "cdylib", unsafe(no_mangle))]
pub unsafe extern "C" fn install(request: *const CBootPluginInstallRequest) -> i32 {
    if request.is_null() {
        return error_code!(UkiError::InvalidRequest);
    }

    let result = BootPluginInstallRequest::try_from(unsafe { &*request })
        .map_err(UkiError::from)
        .and_then(|request| {
            Uki::new().and_then(|mut uki| {
                uki.install(
                    &request.esp_mount_point,
                    request.esp_partition_number,
                    request.esp_starting_lba,
                    request.esp_ending_lba,
                    request.esp_unique_partition_guid,
                    &request.to_slot,
                    &request.from_slot,
                )
            })
        });

    match result {
        Ok(()) => 0,

        Err(error) => error_code!(error),
    }
}
