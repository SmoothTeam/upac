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

use upac_abi::BOOT_ABI_VERSION;
use upac_abi::boot::CBootPluginRequest;
use upac_abi::types::CBorrowed;

const BOOT_NEXT_VAR: &str = "BootNext";

// Only used negatively here (see `probes` below) — systemd-boot's own plugin owns this GUID for
// its actual `LoaderEntryOneShot`/`LoaderEntryDefault` writes.
const SD_BOOT_LOADER_GUID: &str = "4a67b082-0a4c-41cf-b6c7-440b29bb8c4f";

fn entry_name_from_request(request: &CBootPluginRequest) -> Result<String, ()> {
    let bytes = unsafe { request.entry_name.as_borrowed() };

    std::str::from_utf8(bytes).map(str::to_owned).map_err(|_| ())
}

fn find_boot_id(manager: &dyn VarManager, slot_filename: &str) -> Result<u16, ()> {
    for (entry, _var) in manager.get_boot_entries().map_err(|_| ())? {
        let entry = entry.map_err(|_| ())?;
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

    Err(())
}

fn set_one_shot_impl(entry_name: &str) -> Result<(), ()> {
    let mut manager = catch_unwind(AssertUnwindSafe(efivar::system)).map_err(|_| ())?;
    let id = find_boot_id(manager.as_ref(), entry_name)?;

    manager
        .write(
            &Variable::new(BOOT_NEXT_VAR),
            VariableFlags::default(),
            &id.to_le_bytes(),
        )
        .map_err(|_| ())
}

fn confirm_boot_impl(entry_name: &str) -> Result<(), ()> {
    let mut manager = catch_unwind(AssertUnwindSafe(efivar::system)).map_err(|_| ())?;
    let id = find_boot_id(manager.as_ref(), entry_name)?;

    let mut order = manager.get_boot_order().map_err(|_| ())?;
    order.retain(|&existing| existing != id);
    order.insert(0, id);

    manager.set_boot_order(order).map_err(|_| ())
}

fn probes_impl() -> bool {
    if !Path::new("/sys/firmware/efi").exists() {
        return false;
    }

    let Ok(manager) = catch_unwind(AssertUnwindSafe(efivar::system)) else {
        return false;
    };
    let Ok(guid) = Uuid::from_str(SD_BOOT_LOADER_GUID) else {
        return false;
    };

    !manager
        .exists(&Variable::new_with_vendor("LoaderInfo", guid))
        .unwrap_or(false)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn abi_version() -> u32 {
    BOOT_ABI_VERSION
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn probe() -> i32 {
    i32::from(probes_impl())
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn set_one_shot(request: *const CBootPluginRequest) -> i32 {
    if request.is_null() {
        return 1;
    }

    let result = entry_name_from_request(unsafe { &*request }).and_then(|entry_name| set_one_shot_impl(&entry_name));

    match result {
        Ok(()) => 0,
        Err(()) => 1,
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn confirm_boot(request: *const CBootPluginRequest) -> i32 {
    if request.is_null() {
        return 1;
    }

    let result = entry_name_from_request(unsafe { &*request }).and_then(|entry_name| confirm_boot_impl(&entry_name));

    match result {
        Ok(()) => 0,
        Err(()) => 1,
    }
}
