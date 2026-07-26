use std::panic::{AssertUnwindSafe, catch_unwind};

use upac_abi::error::{CError, ErrorDomain, ErrorKind};
use upac_abi::request::CCommitRequest;
use upac_abi::types::AbiError;

use crate::commit::CommitData;
use crate::types::states::CommitStateId;

macro_rules! try_convert_abi {
    ($expr:expr, $err_out:expr) => {
        match $expr {
            Ok(value) => value,
            Err(error) => return write_abi_error(error, $err_out),
        }
    };
}

unsafe fn write_error(err_out: *mut CError, state: CommitStateId, error: ErrorKind) {
    if !err_out.is_null() {
        unsafe {
            *err_out = CError {
                domain: ErrorDomain::Commit,
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

    unsafe { write_error(err_out, CommitStateId::Verifying, kind) };
    -1
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn commit(request_c: CCommitRequest, err_out: *mut CError) -> i32 {
    if let Err(error) = unsafe { request_c.validate() } {
        return write_abi_error(error, err_out);
    }

    let cancel_token = match unsafe { request_c.base.hook_cancel_token.as_ref() } {
        Some(token) => token,
        None => return write_abi_error(AbiError::InvalidEntry, err_out),
    };

    let branch = try_convert_abi!(unsafe { request_c.base.branch.as_str() }, err_out);
    let tmp_path = try_convert_abi!(unsafe { request_c.base.tmp_path.as_str() }, err_out);
    let message = try_convert_abi!(unsafe { request_c.message.as_str() }, err_out);

    let commit_data = CommitData {
        message,
        branch,

        tmp_path,

        hook_message: request_c.base.on_hook,
        hook_message_context: request_c.base.hook_ctx,
        hook_cancel_token: cancel_token,
    };

    let result = catch_unwind(AssertUnwindSafe(|| crate::commit::run(commit_data)));

    match result {
        Ok(Ok(())) => 0,
        Ok(Err((state, error))) => {
            unsafe { write_error(err_out, state, ErrorKind::from(error)) };
            -1
        }
        Err(_) => {
            unsafe { write_error(err_out, CommitStateId::Verifying, ErrorKind::Unexpected) };
            -1
        }
    }
}
