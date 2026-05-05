// ── Imports ─────────────────────────────────────────────────────────────────────
const ffi = @import("upac-ffi");

const CancelToken = ffi.CancelToken;

pub fn get_abi_version() callconv(.c) u32 {
    return ffi.ABI_VERSION;
}

pub fn cancel(token: *CancelToken) callconv(.c) void {
    token.cancel();
}

// Finalizes the allocator and outputs a warning to the console if any memory leaks were detected during program execution
pub fn deinit() callconv(.c) void {
    ffi.deinit();
}
