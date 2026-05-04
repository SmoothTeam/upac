// ── Imports ─────────────────────────────────────────────────────────────────────
const CPackageMeta = ffi.CPackageMeta;
const CCommitEntry = ffi.CCommitEntry;
const ListStateId = ffi.ListStateId;

const CancelToken = ffi.CancelToken;
const cancelGCancellable = ffi.cancelGCancellable;

const states = @import("states.zig");
const stateFailed = states.stateFailed;

// ── Public imports ───────────────────────────────────────────────────────────
pub const std = @import("std");
pub const ffi = @import("upac-ffi");
pub const c_libs = ffi.c_libs;

pub const data = @import("upac-data");

pub const ListError = error{
    RepoOpenFailed,
    CommitNotFound,
    AllocFailed,
    ListError,
    Cancelled,
    MaxRetriesExceeded,
};

pub const ListData = struct {
    repo_path: [*:0]const u8,
    branch: [*:0]const u8,
    db_path: []const u8,

    cancel_token: *CancelToken,
};

pub const ListMachine = struct {
    data: ListData,
    repo: ?*c_libs.OstreeRepo = null,

    result_packages: ?[]CPackageMeta = null,
    result_commits: ?[]CCommitEntry = null,

    cancellable: ?*c_libs.GCancellable = null,
    gerror: ?*c_libs.GError = null,

    stack: std.ArrayList(ListStateId),
    allocator: std.mem.Allocator,

    pub fn enter(self: *ListMachine, state_id: ListStateId) ListError!void {
        isBroked(self) catch |err| return err;

        self.stack.append(self.allocator, state_id) catch return ListError.AllocFailed;
    }

    fn isBroked(self: *ListMachine) ListError!void {
        errdefer stateFailed(self);

        if (self.gerror) |err| {
            const is_cancel = err.domain == c_libs.g_io_error_quark() and
                err.code == c_libs.G_IO_ERROR_CANCELLED;

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

    pub inline fn unwrap(self: *ListMachine, value: anytype, comptime err: ListError) ListError!@typeInfo(@TypeOf(value)).optional.child {
        return value orelse {
            stateFailed(self);
            return err;
        };
    }

    pub inline fn check(self: *ListMachine, value: anytype, comptime err: ListError) ListError!@typeInfo(@TypeOf(value)).error_union.payload {
        return value catch {
            stateFailed(self);
            return err;
        };
    }

    pub fn deinit(self: *ListMachine) void {
        if (self.repo) |repo| c_libs.g_object_unref(repo);
        if (self.gerror) |err| c_libs.g_error_free(err);
        if (self.cancellable) |cancellable| c_libs.g_object_unref(cancellable);

        self.stack.deinit(self.allocator);
    }

    pub fn runPackages(list_data: ListData, allocator: std.mem.Allocator) ListError![]CPackageMeta {
        var machine = ListMachine{
            .data = list_data,

            .cancellable = c_libs.g_cancellable_new() orelse return ListError.Cancelled,
            .stack = std.ArrayList(ListStateId).empty,
            .allocator = allocator,
        };
        defer machine.deinit();

        list_data.cancel_token.hook = cancelGCancellable;
        list_data.cancel_token.hook_ctx = machine.cancellable;
        defer list_data.cancel_token.reset();

        try machine.enter(.open_repo);
        try states.stateOpenRepo(&machine);
        try machine.enter(.list_packages);
        try states.stateListPackages(&machine);
        try machine.enter(.done);

        return machine.result_packages orelse &.{};
    }

    pub fn runCommits(list_data: ListData, allocator: std.mem.Allocator) ListError![]CCommitEntry {
        var machine = ListMachine{
            .data = list_data,

            .cancellable = c_libs.g_cancellable_new() orelse return ListError.Cancelled,
            .stack = std.ArrayList(ListStateId).empty,
            .allocator = allocator,
        };
        defer machine.deinit();

        list_data.cancel_token.hook = cancelGCancellable;
        list_data.cancel_token.hook_ctx = machine.cancellable;
        defer list_data.cancel_token.reset();

        try machine.enter(.open_repo);
        try states.stateOpenRepo(&machine);
        try machine.enter(.list_commits);
        try states.stateListCommits(&machine);
        try machine.enter(.done);

        return machine.result_commits orelse &.{};
    }
};
