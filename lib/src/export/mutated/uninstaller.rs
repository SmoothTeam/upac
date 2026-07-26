use std::panic::{AssertUnwindSafe, catch_unwind};
use std::slice::from_raw_parts;

use upac_abi::error::{CError, ErrorDomain, ErrorKind};
use upac_abi::request::CUninstallRequest;
use upac_abi::types::AbiError;

use crate::types::states::UninstallStateId;
use crate::uninstaller::{UninstallData, UninstallPackage};

macro_rules! try_convert_abi {
    ($expr:expr, $err_out:expr) => {
        match $expr {
            Ok(value) => value,
            Err(error) => return write_abi_error(error, $err_out),
        }
    };
}

unsafe fn write_error(err_out: *mut CError, state: UninstallStateId, error: ErrorKind) {
    if !err_out.is_null() {
        unsafe {
            *err_out = CError {
                domain: ErrorDomain::Uninstall,
                state: state as u32,
                error,
            };
        }
    }
}

fn write_abi_error(error: AbiError, err_out: *mut CError) -> i32 {
    let kind = match error {
        AbiError::AbiMismatch => ErrorKind::AbiMismatch,
        AbiError::InvalidEntry => ErrorKind::InvalidEntry,
    };

    unsafe { write_error(err_out, UninstallStateId::Setup, kind) };
    -1
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn uninstall(request_c: CUninstallRequest, err_out: *mut CError) -> i32 {
    if let Err(error) = unsafe { request_c.validate() } {
        return write_abi_error(error, err_out);
    }

    let packages_c = unsafe { from_raw_parts(request_c.packages, request_c.packages_count) };

    let mut packages = Vec::with_capacity(packages_c.len());
    for package_info_c in packages_c {
        let name = try_convert_abi!(unsafe { package_info_c.name.as_str() }, err_out);
        let arch = try_convert_abi!(unsafe { package_info_c.arch.as_str() }, err_out);
        let arch_sub = if package_info_c.arch_sub.ptr.is_null() {
            None
        } else {
            Some(try_convert_abi!(unsafe { package_info_c.arch_sub.as_str() }, err_out))
        };

        packages.push(UninstallPackage { name, arch, arch_sub });
    }

    let cancel_token = match unsafe { request_c.base.hook_cancel_token.as_ref() } {
        Some(token) => token,
        None => return write_abi_error(AbiError::InvalidEntry, err_out),
    };

    let branch = try_convert_abi!(unsafe { request_c.base.branch.as_str() }, err_out);
    let tmp_path = try_convert_abi!(unsafe { request_c.base.tmp_path.as_str() }, err_out);

    let uninstall_data = UninstallData {
        packages: &packages,
        branch,

        tmp_path,

        hook_message: request_c.base.on_hook,
        hook_message_context: request_c.base.hook_ctx,
        hook_cancel_token: cancel_token,
    };

    let result = catch_unwind(AssertUnwindSafe(|| crate::uninstaller::run(uninstall_data)));

    match result {
        Ok(Ok(())) => 0,
        Ok(Err((state, error))) => {
            unsafe { write_error(err_out, state, ErrorKind::from(error)) };
            -1
        }
        Err(_) => {
            unsafe { write_error(err_out, UninstallStateId::Setup, ErrorKind::Unexpected) };
            -1
        }
    }
}
