use std::panic::{AssertUnwindSafe, catch_unwind};

use upac_abi::error::{CError, ErrorKind};
use upac_abi::request::CFilesRequest;

use crate::export::{try_convert_abi, write_error};
use crate::mutated::files::FilesData;
use crate::types::states::FilesStateId;

#[unsafe(no_mangle)]
pub unsafe extern "C" fn files(request_c: CFilesRequest, err_out: *mut CError) -> i32 {
    let files_data = try_convert_abi!(FilesData::try_from(&request_c), err_out, FilesStateId);

    let result = catch_unwind(AssertUnwindSafe(|| crate::mutated::files::run(files_data)));

    match result {
        Ok(Ok(())) => 0,
        Ok(Err((state, error))) => {
            unsafe { write_error(err_out, state, ErrorKind::from(error)) };
            -1
        }
        Err(_) => {
            unsafe { write_error(err_out, FilesStateId::Setup, ErrorKind::Unexpected) };
            -1
        }
    }
}
