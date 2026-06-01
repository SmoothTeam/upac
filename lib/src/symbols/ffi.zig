// ── Imports ─────────────────────────────────────────────────────────────────────
const ffi = @import("upac-ffi");

const CancelToken = ffi.CancelToken;
const CUnmutatedResponse = ffi.CUnmutatedResponse;

pub fn version_abi() callconv(.c) u32 {
    return ffi.ABI_VERSION;
}

pub fn cancel(token: *CancelToken) callconv(.c) void {
    token.cancel();
}

pub fn free_response(response: *CUnmutatedResponse) callconv(.c) void {
    response.free(ffi.getAllocator());
}
