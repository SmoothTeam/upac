// ── Imports ─────────────────────────────────────────────────────────────────────
pub const std = @import("std");

pub const ffi = @import("upac-ffi");
const c_libs = ffi.c_libs;

const CPackageMeta = ffi.CPackageMeta;
const CCommitEntry = ffi.CCommitEntry;

const CancelToken = ffi.CancelToken;
const cancelGCancellable = ffi.cancelGCancellable;

pub const types = @import("upac-types");
const ListStateId = types.ListStateId;

pub const database = @import("upac-database");

const states = @import("states.zig");
const stateFailed = states.stateFailed;

pub const ListError = error{
    RepoOpenFailed,
    CommitNotFound,
    AllocFailed,
    OutOfMemory,
    ListError,
    Cancelled,
    MaxRetriesExceeded,
};

pub const ListData = struct {
    repo_path: [*:0]const u8,
    branch: [*:0]const u8,
    root_path: [*:0]const u8,

    cancel_token: *CancelToken,
};

pub const ListMachine = struct {
    data: ListData,
    repo: ?*c_libs.OstreeRepo = null,

    cancellable: ?*c_libs.GCancellable = null,
    gerror: ?*c_libs.GError = null,

    allocator: std.mem.Allocator,

    pub fn check(self: *ListMachine) ListError!void {
        if (self.gerror) |err| {
            const is_cancel = err.domain == c_libs.g_io_error_quark() and err.code == c_libs.G_IO_ERROR_CANCELLED;

            c_libs.g_error_free(err);
            self.gerror = null;

            return if (is_cancel) ListError.Cancelled else ListError.ListError;
        }

        if (self.cancellable) |cancellable| {
            if (c_libs.g_cancellable_is_cancelled(cancellable) != 0) {
                c_libs.g_cancellable_cancel(cancellable);
                return ListError.Cancelled;
            }
        }
    }

    pub fn deinit(self: *ListMachine) void {
        if (self.repo) |repo| c_libs.g_object_unref(repo);
        if (self.gerror) |err| c_libs.g_error_free(err);
        if (self.cancellable) |cancellable| c_libs.g_object_unref(cancellable);
    }

    pub fn runPackages(list_data: ListData, allocator: std.mem.Allocator) ListError![]CPackageMeta {
        var machine = ListMachine{
            .data = list_data,

            .cancellable = c_libs.g_cancellable_new() orelse return ListError.Cancelled,

            .allocator = allocator,
        };
        defer machine.deinit();

        list_data.cancel_token.hook = cancelGCancellable;
        list_data.cancel_token.hook_ctx = machine.cancellable;
        defer list_data.cancel_token.reset();

        try states.stateOpenRepo(&machine);
        return states.stateListPackages(&machine);
    }

    pub fn runCommits(list_data: ListData, allocator: std.mem.Allocator) ListError![]CCommitEntry {
        var machine = ListMachine{
            .data = list_data,

            .cancellable = c_libs.g_cancellable_new() orelse return ListError.Cancelled,

            .allocator = allocator,
        };
        defer machine.deinit();

        list_data.cancel_token.hook = cancelGCancellable;
        list_data.cancel_token.hook_ctx = machine.cancellable;
        defer list_data.cancel_token.reset();

        try states.stateOpenRepo(&machine);
        return states.stateListCommits(&machine);
    }
};
