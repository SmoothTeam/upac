use std::panic::{AssertUnwindSafe, catch_unwind};

use upac_abi::error::{CError, ErrorKind};
use upac_abi::request::CUninstallRequest;

use crate::export::{try_convert_abi, write_error};
use crate::mutated::uninstaller::UninstallData;
use crate::types::states::UninstallStateId;

#[unsafe(no_mangle)]
pub unsafe extern "C" fn uninstall(request_c: CUninstallRequest, err_out: *mut CError) -> i32 {
    let uninstall_data = try_convert_abi!(UninstallData::try_from(&request_c), err_out, UninstallStateId);

    let result = catch_unwind(AssertUnwindSafe(|| crate::mutated::uninstaller::run(uninstall_data)));

    match result {
        Ok(Ok(())) => 0,
        Ok(Err((state, error))) => {
            unsafe { write_error(err_out, state, ErrorKind::from(error)) };
            -1
        }
        Err(_) => {
            unsafe { write_error(err_out, UninstallStateId::Setup, ErrorKind::Unexpected) };
            -1
        }
    }
}
