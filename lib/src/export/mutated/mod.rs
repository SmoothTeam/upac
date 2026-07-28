use upac_abi::error::{CError, CommandState, ErrorKind};

pub mod commit;
pub mod files;
pub mod installer;
pub mod rollback;
pub mod uninstaller;
pub mod update;

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
            Err(error) => return crate::export::mutated::write_abi_error::<$state>(error, $err_out),
        }
    };
}
pub(crate) use try_convert_abi;
