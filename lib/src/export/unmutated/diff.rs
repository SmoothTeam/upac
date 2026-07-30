use std::mem::size_of;
use std::panic::{AssertUnwindSafe, catch_unwind};

use upac_abi::error::{CError, ErrorKind};
use upac_abi::request::CDiffRequest;
use upac_abi::response::{CDiffFileEntry, CDiffPackageEntry, CDiffResponse};
use upac_abi::types::{COwned, CVec};

use crate::export::{try_convert_abi, write_error};
use crate::types::states::DiffStateId;
use crate::unmutated::diff::DiffData;

#[unsafe(no_mangle)]
pub unsafe extern "C" fn diff(request_c: CDiffRequest, response_out: *mut CDiffResponse, err_out: *mut CError) -> i32 {
    let diff_data = try_convert_abi!(DiffData::try_from(&request_c), err_out, DiffStateId);

    let result = catch_unwind(AssertUnwindSafe(|| crate::unmutated::diff::run(diff_data)));

    match result {
        Ok(Ok((files, diff_packages))) => {
            if !response_out.is_null() {
                unsafe {
                    *response_out = CDiffResponse {
                        struct_size: size_of::<CDiffResponse>(),
                        files: CVec::from_owned(files.into_iter().map(CDiffFileEntry::from).collect()),
                        diff_packages: CVec::from_owned(diff_packages.into_iter().map(CDiffPackageEntry::from).collect()),
                    };
                }
            }
            0
        }
        Ok(Err((state, error))) => {
            unsafe { write_error(err_out, state, ErrorKind::from(error)) };
            -1
        }
        Err(_) => {
            unsafe { write_error(err_out, DiffStateId::Setup, ErrorKind::Unexpected) };
            -1
        }
    }
}
