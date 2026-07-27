use std::panic::{AssertUnwindSafe, catch_unwind};

use upac_abi::error::{CError, ErrorDomain, ErrorKind};
use upac_abi::request::CFilesRequest;

use crate::files::FilesData;
use crate::types::states::FilesStateId;

macro_rules! try_convert_abi {
    ($expr:expr, $err_out:expr) => {
        match $expr {
            Ok(value) => value,
            Err(error) => return write_abi_error(error, $err_out),
        }
    };
}

unsafe fn write_error(err_out: *mut CError, state: FilesStateId, error: ErrorKind) {
    if !err_out.is_null() {
        unsafe {
            *err_out = CError {
                domain: ErrorDomain::Files,
                state: state as u32,
                error,
            };
        }
    }
}

fn write_abi_error(error: ErrorKind, err_out: *mut CError) -> i32 {
    unsafe { write_error(err_out, FilesStateId::Verifying, error) };
    -1
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn files(request_c: CFilesRequest, err_out: *mut CError) -> i32 {
    let files_data = try_convert_abi!(FilesData::try_from(&request_c), err_out);

    let result = catch_unwind(AssertUnwindSafe(|| crate::files::run(files_data)));

    match result {
        Ok(Ok(())) => 0,
        Ok(Err((state, error))) => {
            unsafe { write_error(err_out, state, ErrorKind::from(error)) };
            -1
        }
        Err(_) => {
            unsafe { write_error(err_out, FilesStateId::Verifying, ErrorKind::Unexpected) };
            -1
        }
    }
}
