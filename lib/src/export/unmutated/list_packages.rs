use std::mem::size_of;
use std::panic::{AssertUnwindSafe, catch_unwind};

use upac_abi::error::{CError, ErrorKind};
use upac_abi::package::CPackageMeta;
use upac_abi::request::CListPackagesRequest;
use upac_abi::response::CListPackagesResponse;
use upac_abi::types::{COwned, CVec};

use crate::export::{try_convert_abi, write_error};
use crate::types::states::ListPackagesStateId;
use crate::unmutated::list_packages::ListPackagesData;

#[unsafe(no_mangle)]
pub unsafe extern "C" fn list_packages(
    request_c: CListPackagesRequest, response_out: *mut CListPackagesResponse, err_out: *mut CError,
) -> i32 {
    let list_packages_data = try_convert_abi!(ListPackagesData::try_from(&request_c), err_out, ListPackagesStateId);

    let result = catch_unwind(AssertUnwindSafe(|| {
        crate::unmutated::list_packages::run(list_packages_data)
    }));

    match result {
        Ok(Ok(metas)) => {
            if !response_out.is_null() {
                unsafe {
                    *response_out = CListPackagesResponse {
                        struct_size: size_of::<CListPackagesResponse>(),
                        metas: CVec::from_owned(metas.into_iter().map(CPackageMeta::from).collect()),
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
            unsafe { write_error(err_out, ListPackagesStateId::Setup, ErrorKind::Unexpected) };
            -1
        }
    }
}
