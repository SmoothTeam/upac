use std::mem::size_of;
use std::panic::{AssertUnwindSafe, catch_unwind};

use upac_abi::error::{CError, ErrorKind};
use upac_abi::request::CDiffPackagesRequest;
use upac_abi::response::{CDiffPackageEntry, CDiffPackagesResponse};
use upac_abi::types::{COwned, CVec};

use crate::export::{try_convert_abi, write_error};
use crate::types::states::DiffPackagesStateId;
use crate::unmutated::diff_packages::DiffPackagesData;

#[unsafe(no_mangle)]
pub unsafe extern "C" fn diff_packages(request_c: CDiffPackagesRequest, response_out: *mut CDiffPackagesResponse, err_out: *mut CError) -> i32 {
    let diff_packages_data = try_convert_abi!(DiffPackagesData::try_from(&request_c), err_out, DiffPackagesStateId);

    let result = catch_unwind(AssertUnwindSafe(|| crate::unmutated::diff_packages::run(diff_packages_data)));

    match result {
        Ok(Ok(diff_packages)) => {
            if !response_out.is_null() {
                unsafe {
                    *response_out = CDiffPackagesResponse {
                        struct_size: size_of::<CDiffPackagesResponse>(),
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
            unsafe { write_error(err_out, DiffPackagesStateId::Setup, ErrorKind::Unexpected) };
            -1
        }
    }
}
