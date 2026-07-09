use std::panic::{AssertUnwindSafe, catch_unwind};
use std::slice::from_raw_parts;

use crate::ffi::{AbiError, CMutatedRequest};
use crate::types::errors::ErrorCode;
use crate::uninstaller::{UninstallData, UninstallPackage};

macro_rules! try_convert_abi {
    ($expr:expr) => {
        match $expr {
            Ok(value) => value,
            Err(error) => return error_to_code(error) as i32,
        }
    };
}

fn error_to_code(error: AbiError) -> ErrorCode {
    match error {
        AbiError::AbiMismatch => ErrorCode::AbiMismatch,
        AbiError::InvalidEntry => ErrorCode::DbInvalidEntry,
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn uninstall(request_c: CMutatedRequest) -> i32 {
    if let Err(error) = unsafe { request_c.validate() } {
        return match error {
            AbiError::AbiMismatch => ErrorCode::AbiMismatch,
            AbiError::InvalidEntry => ErrorCode::DbInvalidEntry,
        } as i32;
    }

    if request_c.uninstall_packages.is_null() || request_c.uninstall_packages_len == 0 {
        return ErrorCode::DbInvalidEntry as i32;
    }

    let packages_c = unsafe { from_raw_parts(request_c.uninstall_packages, request_c.uninstall_packages_len) };

    for package_info_c in packages_c {
        if let Err(error) = unsafe { package_info_c.validate() } {
            return match error {
                AbiError::AbiMismatch => ErrorCode::AbiMismatch,
                AbiError::InvalidEntry => ErrorCode::DbInvalidEntry,
            } as i32;
        }
    }

    let mut packages = Vec::with_capacity(packages_c.len());
    for package_info_c in packages_c {
        let name = try_convert_abi!(unsafe { package_info_c.name.as_str() });
        let arch = try_convert_abi!(unsafe { package_info_c.arch.as_str() });
        let arch_sub = if package_info_c.arch_sub.ptr.is_null() {
            None
        } else {
            Some(try_convert_abi!(unsafe { package_info_c.arch_sub.as_str() }))
        };

        packages.push(UninstallPackage { name, arch, arch_sub });
    }

    let cancel_token = match unsafe { request_c.hook_cancel_token.as_ref() } {
        Some(token) => token,
        None => return ErrorCode::DbInvalidEntry as i32,
    };

    let branch = try_convert_abi!(unsafe { request_c.branch.as_str() });
    let repo_path = try_convert_abi!(unsafe { request_c.repo_path.as_str() });
    let root_path = try_convert_abi!(unsafe { request_c.root_path.as_str() });
    let tmp_path = try_convert_abi!(unsafe { request_c.tmp_path.as_str() });

    let uninstall_data = UninstallData {
        packages: &packages,
        branch,

        repo_path,
        root_path,
        tmp_path,

        hook_message: request_c.on_hook,
        hook_message_context: request_c.hook_ctx,
        hook_cancel_token: cancel_token,
    };

    let result = catch_unwind(AssertUnwindSafe(|| crate::uninstaller::run(uninstall_data)));

    match result {
        Ok(code) => code,
        Err(_) => ErrorCode::Unexpected as i32,
    }
}
