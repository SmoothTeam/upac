use crate::ffi::{ABI_VERSION, CUnmutatedResponse, HookCancelToken};

mod mutated;

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
pub unsafe extern "C" fn free_unmutated_response(response: *mut CUnmutatedResponse) {
    if response.is_null() {
        return;
    }
    unsafe { (*response).free() };
}
