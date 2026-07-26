use std::panic::{AssertUnwindSafe, catch_unwind};

use upac_abi::request::CFilesRequest;
use upac_abi::types::{AbiError, CBorrowed};
use upac_abi::error::{CError, ErrorDomain, ErrorKind};

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

fn write_abi_error(error: AbiError, err_out: *mut CError) -> i32 {
    let kind = match error {
        AbiError::AbiMismatch => ErrorKind::AbiMismatch,
        AbiError::InvalidEntry => ErrorKind::InvalidEntry,
    };

    unsafe { write_error(err_out, FilesStateId::Verifying, kind) };
    -1
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn files(request_c: CFilesRequest, err_out: *mut CError) -> i32 {
    if let Err(error) = unsafe { request_c.validate() } {
        return write_abi_error(error, err_out);
    }

    let files_c = unsafe { request_c.files.as_borrowed() };

    let mut files = Vec::with_capacity(files_c.len());
    for file_c in files_c {
        files.push(try_convert_abi!(unsafe { file_c.as_str() }, err_out));
    }

    let file_package = match unsafe { request_c.file_package.as_ref() } {
        Some(package) => package,
        None => return write_abi_error(AbiError::InvalidEntry, err_out),
    };

    let cancel_token = match unsafe { request_c.base.hook_cancel_token.as_ref() } {
        Some(token) => token,
        None => return write_abi_error(AbiError::InvalidEntry, err_out),
    };

    let branch = try_convert_abi!(unsafe { request_c.base.branch.as_str() }, err_out);
    let tmp_path = try_convert_abi!(unsafe { request_c.base.tmp_path.as_str() }, err_out);

    let files_data = FilesData {
        files: &files,
        file_kind: request_c.file_kind,
        file_package,

        branch,
        tmp_path,

        hook_message: request_c.base.on_hook,
        hook_message_context: request_c.base.hook_ctx,
        hook_cancel_token: cancel_token,
    };

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
