use upac_abi::ABI_VERSION;
use upac_abi::error::{CError, CommandState, ErrorKind};
use upac_abi::hook::HookCancelToken;
use upac_abi::response::CUnmutatedResponse;

pub mod mutated;
pub mod unmutated;

#[unsafe(no_mangle)]
pub unsafe extern "C" fn version_abi() -> u32 {
    ABI_VERSION
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn cancel(token: *mut HookCancelToken) {
    if token.is_null() {
        return;
    }
    unsafe { (*token).cancel() };
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn free_response(response: *mut CUnmutatedResponse) {
    if response.is_null() {
        return;
    }
    unsafe { (*response).free() };
}

pub(crate) unsafe fn write_error<S: CommandState>(err_out: *mut CError, state: S, error: ErrorKind) {
    if !err_out.is_null() {
        unsafe {
            *err_out = CError {
                domain: S::DOMAIN,
                state: state.as_u32(),
                error,
            };
        }
    }
}

pub(crate) fn write_abi_error<S: CommandState>(error: ErrorKind, err_out: *mut CError) -> i32 {
    unsafe { write_error(err_out, S::VALIDATION, error) };
    -1
}

macro_rules! try_convert_abi {
    ($expr:expr, $err_out:expr, $state:ty) => {
        match $expr {
            Ok(value) => value,
            Err(error) => return crate::export::write_abi_error::<$state>(error, $err_out),
        }
    };
}
pub(crate) use try_convert_abi;
