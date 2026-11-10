// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_abi::error::CError;
use upac_abi::request::mutated::{
    CAttachRequest, CCommitRequest, CDetachRequest, CGcRequest, CInstallRequest, CMimeSyncRequest, CPinRequest,
    CRollbackRequest, CUninstallRequest, CUpdateRequest,
};

use upac_types::error::export_mutated_command;

use crate::mutated::{attach, commit, detach, gc, installer, mime, pin, rollback, uninstaller, update};

/// # Safety
/// Any borrowed byte-slice fields inside `request` must remain valid for the duration of the call.
/// `err_out`, if non-null, must point to writable `CError` storage.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn install(request: CInstallRequest, err_out: *mut CError) -> i32 {
    unsafe { export_mutated_command(&request, err_out, installer::run) }
}

/// # Safety
/// Any borrowed byte-slice fields inside `request` must remain valid for the duration of the call.
/// `err_out`, if non-null, must point to writable `CError` storage.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn update(request: CUpdateRequest, err_out: *mut CError) -> i32 {
    unsafe { export_mutated_command(&request, err_out, update::run) }
}

/// # Safety
/// Any borrowed byte-slice fields inside `request` must remain valid for the duration of the call.
/// `err_out`, if non-null, must point to writable `CError` storage.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn uninstall(request: CUninstallRequest, err_out: *mut CError) -> i32 {
    unsafe { export_mutated_command(&request, err_out, uninstaller::run) }
}

/// # Safety
/// Any borrowed byte-slice fields inside `request` must remain valid for the duration of the call.
/// `err_out`, if non-null, must point to writable `CError` storage.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn attach(request: CAttachRequest, err_out: *mut CError) -> i32 {
    unsafe { export_mutated_command(&request, err_out, attach::run) }
}

/// # Safety
/// Any borrowed byte-slice fields inside `request` must remain valid for the duration of the call.
/// `err_out`, if non-null, must point to writable `CError` storage.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn detach(request: CDetachRequest, err_out: *mut CError) -> i32 {
    unsafe { export_mutated_command(&request, err_out, detach::run) }
}

/// # Safety
/// Any borrowed byte-slice fields inside `request` must remain valid for the duration of the call.
/// `err_out`, if non-null, must point to writable `CError` storage.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rollback(request: CRollbackRequest, err_out: *mut CError) -> i32 {
    unsafe { export_mutated_command(&request, err_out, rollback::run) }
}

/// # Safety
/// Any borrowed byte-slice fields inside `request` must remain valid for the duration of the call.
/// `err_out`, if non-null, must point to writable `CError` storage.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn commit(request: CCommitRequest, err_out: *mut CError) -> i32 {
    unsafe { export_mutated_command(&request, err_out, commit::run) }
}

/// # Safety
/// Any borrowed byte-slice fields inside `request` must remain valid for the duration of the call.
/// `err_out`, if non-null, must point to writable `CError` storage.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pin_deploy(request: CPinRequest, err_out: *mut CError) -> i32 {
    unsafe { export_mutated_command(&request, err_out, pin::run) }
}

/// # Safety
/// Any borrowed byte-slice fields inside `request` must remain valid for the duration of the call.
/// `err_out`, if non-null, must point to writable `CError` storage.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn gc(request: CGcRequest, err_out: *mut CError) -> i32 {
    unsafe { export_mutated_command(&request, err_out, gc::run) }
}

/// # Safety
/// Any borrowed byte-slice fields inside `request` must remain valid for the duration of the call.
/// `err_out`, if non-null, must point to writable `CError` storage.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mime(request: CMimeSyncRequest, err_out: *mut CError) -> i32 {
    unsafe { export_mutated_command(&request, err_out, mime::run) }
}
