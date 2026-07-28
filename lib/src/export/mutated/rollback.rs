use std::panic::{AssertUnwindSafe, catch_unwind};

use upac_abi::error::{CError, ErrorKind};
use upac_abi::request::CRollbackRequest;

use crate::export::{try_convert_abi, write_error};
use crate::mutated::rollback::RollbackData;
use crate::types::states::RollbackStateId;

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rollback(request_c: CRollbackRequest, err_out: *mut CError) -> i32 {
    let rollback_data = try_convert_abi!(RollbackData::try_from(&request_c), err_out, RollbackStateId);

    let result = catch_unwind(AssertUnwindSafe(|| crate::mutated::rollback::run(rollback_data)));

    match result {
        Ok(Ok(())) => 0,
        Ok(Err((state, error))) => {
            unsafe { write_error(err_out, state, ErrorKind::from(error)) };
            -1
        }
        Err(_) => {
            unsafe { write_error(err_out, RollbackStateId::Setup, ErrorKind::Unexpected) };
            -1
        }
    }
}
