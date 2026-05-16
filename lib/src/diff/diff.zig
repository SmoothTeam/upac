// ── Imports ──────────────────────────────────────────────────────────
const std = @import("std");

const c_libs = @import("c-libs");

const ffi = @import("upac-ffi");

const CDiffEntry = ffi.CDiffEntry;

const CancelToken = ffi.CancelToken;
const cancelGCancellable = ffi.cancelGCancellable;

const states = @import("states.zig");
const stateFailed = states.stateFailed;
// ── Errors ───────────────────────────────────────────────────────────────────
pub const DiffError = error{
    RepoOpenFailed,
    CommitNotFound,
    DiffFailed,
    AllocFailed,
    OutOfMemory,
    FileNotFound,
    Cancelled,
};

pub const DiffData = struct {
    repo_path: [*:0]const u8,
    root_path: [*:0]const u8,

    from_ref: [*:0]const u8,
    to_ref: [*:0]const u8,

    cancel_token: *CancelToken,
};

pub const DiffMachine = struct {
    data: DiffData,
    repo: ?*c_libs.OstreeRepo = null,

    cancellable: ?*c_libs.GCancellable = null,
    gerror: ?*c_libs.GError = null,

    allocator: std.mem.Allocator,

    pub fn check(self: *DiffMachine) DiffError!void {
        if (self.gerror) |err| {
            const is_cancel = err.domain == c_libs.g_io_error_quark() and err.code == c_libs.G_IO_ERROR_CANCELLED;

            c_libs.g_error_free(err);
            self.gerror = null;

            return if (is_cancel) DiffError.Cancelled else DiffError.DiffFailed;
        }

        if (self.cancellable) |cancellable| {
            if (c_libs.g_cancellable_is_cancelled(cancellable) != 0) {
                c_libs.g_cancellable_cancel(cancellable);
                return DiffError.Cancelled;
            }
        }
    }

    pub fn deinit(self: *DiffMachine) void {
        if (self.repo) |repo| c_libs.g_object_unref(repo);
        if (self.gerror) |err| c_libs.g_error_free(err);
        if (self.cancellable) |cancellable| c_libs.g_object_unref(cancellable);
    }

    pub fn run(diff_data: DiffData, allocator: std.mem.Allocator) DiffError![]CDiffEntry {
        var machine = DiffMachine{
            .data = diff_data,

            .cancellable = c_libs.g_cancellable_new() orelse return DiffError.Cancelled,

            .allocator = allocator,
        };
        defer machine.deinit();

        diff_data.cancel_token.hook = cancelGCancellable;
        diff_data.cancel_token.hook_ctx = machine.cancellable;
        defer diff_data.cancel_token.reset();

        try states.stateOpenRepo(&machine);
        return states.stateDiffAttributed(&machine) catch |err| return err;
    }
};
