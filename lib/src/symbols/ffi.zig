// ── Imports ─────────────────────────────────────────────────────────────────────
const ffi = @import("upac-ffi");

const CancelToken = ffi.CancelToken;
const CUnmutatedResponse = ffi.CUnmutatedResponse;

pub fn get_abi_version() callconv(.c) u32 {
    return ffi.ABI_VERSION;
}

pub fn cancel(token: *CancelToken) callconv(.c) void {
    token.cancel();
}

pub fn response_free(response: *CUnmutatedResponse) callconv(.c) void {
    response.free(ffi.getAllocator());
}
