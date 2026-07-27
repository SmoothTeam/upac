use std::panic::{AssertUnwindSafe, catch_unwind};

use upac_abi::error::{CError, ErrorDomain, ErrorKind};
use upac_abi::request::CInstallRequest;

use crate::installer::InstallData;
use crate::types::states::InstallStateId;

macro_rules! try_convert_abi {
    ($expr:expr, $err_out:expr) => {
        match $expr {
            Ok(value) => value,
            Err(error) => return write_abi_error(error, $err_out),
        }
    };
}

unsafe fn write_error(err_out: *mut CError, state: InstallStateId, error: ErrorKind) {
    if !err_out.is_null() {
        unsafe {
            *err_out = CError {
                domain: ErrorDomain::Install,
                state: state as u32,
                error,
            };
        }
    }
}

fn write_abi_error(error: ErrorKind, err_out: *mut CError) -> i32 {
    unsafe { write_error(err_out, InstallStateId::Verifying, error) };
    -1
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn install(request_c: CInstallRequest, err_out: *mut CError) -> i32 {
    let install_data = try_convert_abi!(InstallData::try_from(&request_c), err_out);

    let result = catch_unwind(AssertUnwindSafe(|| crate::installer::run(install_data)));

    match result {
        Ok(Ok(())) => 0,
        Ok(Err((state, error))) => {
            unsafe { write_error(err_out, state, ErrorKind::from(error)) };
            -1
        }
        Err(_) => {
            unsafe { write_error(err_out, InstallStateId::Verifying, ErrorKind::Unexpected) };
            -1
        }
    }
}
