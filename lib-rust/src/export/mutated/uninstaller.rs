use std::os::raw::c_void;
use std::slice::from_raw_parts;

use crate::ffi::{AbiError, CMutatedRequest, CancelToken, HookFn};
use crate::types::errors::ErrorCode;

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

pub struct UninstallPackage<'a> {
    pub name: &'a str,
    pub arch: &'a str,
    pub arch_sub: Option<&'a str>,
}

pub struct UninstallData<'a> {
    pub packages: &'a [UninstallPackage<'a>],
    pub branch: &'a str,
    pub repo_path: &'a str,
    pub root_path: &'a str,
    pub tmp_path: &'a str,
    pub on_hook: Option<HookFn>,
    pub hook_ctx: *mut c_void,
    pub cancel_token: &'a CancelToken,
}

unsafe fn cslice_str<'a>(s: &crate::ffi::CSlice) -> Result<&'a str, AbiError> {
    let bytes = unsafe { from_raw_parts(s.ptr, s.len) };
    std::str::from_utf8(bytes).map_err(|_| AbiError::InvalidEntry)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn upac_uninstall(request_c: CMutatedRequest) -> i32 {
    if let Err(error) = unsafe { request_c.validate() } {
        return match error {
            AbiError::AbiMismatch => ErrorCode::AbiMismatch,
            AbiError::InvalidEntry => ErrorCode::DbInvalidEntry,
        } as i32;
    }

    if request_c.uninstall_packages.is_null() || request_c.uninstall_packages_len == 0 {
        return ErrorCode::DbInvalidEntry as i32;
    }

    let packages_c = unsafe {
        from_raw_parts(
            request_c.uninstall_packages,
            request_c.uninstall_packages_len,
        )
    };

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
        let name = try_convert_abi!(unsafe { cslice_str(&package_info_c.name) });
        let arch = try_convert_abi!(unsafe { cslice_str(&package_info_c.arch) });
        let arch_sub = if package_info_c.arch_sub.ptr.is_null() {
            None
        } else {
            Some(try_convert_abi!(unsafe {
                cslice_str(&package_info_c.arch_sub)
            }))
        };

        packages.push(UninstallPackage {
            name,
            arch,
            arch_sub,
        });
    }

    let cancel_token = match unsafe { request_c.cancel_token.as_ref() } {
        Some(token) => token,
        None => return ErrorCode::DbInvalidEntry as i32,
    };

    let branch = try_convert_abi!(unsafe { cslice_str(&request_c.branch) });
    let repo_path = try_convert_abi!(unsafe { cslice_str(&request_c.repo_path) });
    let root_path = try_convert_abi!(unsafe { cslice_str(&request_c.root_path) });
    let tmp_path = try_convert_abi!(unsafe { cslice_str(&request_c.tmp_path) });

    let uninstall_data = UninstallData {
        packages: &packages,
        branch,
        repo_path,
        root_path,
        tmp_path,
        on_hook: request_c.on_hook,
        hook_ctx: request_c.hook_ctx,
        cancel_token,
    };

    todo!()
}
