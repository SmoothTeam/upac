use upac_abi::ABI_VERSION;
use upac_abi::hook::HookCancelToken;
use upac_abi::request::CUnmutatedResponse;

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
pub unsafe extern "C" fn free_response(response: *mut CUnmutatedResponse) {
    if response.is_null() {
        return;
    }
    unsafe { (*response).free() };
}
