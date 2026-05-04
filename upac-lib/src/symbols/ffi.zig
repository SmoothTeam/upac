// ── Imports ─────────────────────────────────────────────────────────────────────
const ffi = @import("upac-ffi");

pub fn request_cancel() callconv(.c) void {
    if (ffi.active_cancellable.load(.acquire)) |cancellable| ffi.c_libs.g_cancellable_cancel(cancellable);
}

pub fn reset_cancel() callconv(.c) void {}

// Finalizes the allocator and outputs a warning to the console if any memory leaks were detected during program execution
pub fn deinit() callconv(.c) void {
    ffi.deinit();
}
