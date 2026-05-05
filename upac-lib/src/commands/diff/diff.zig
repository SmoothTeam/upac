// ── Imports ──────────────────────────────────────────────────────────
const states = @import("states.zig");
const stateFailed = states.stateFailed;

const CSlice = ffi.CSlice;

const CancelToken = ffi.CancelToken;
const cancelGCancellable = ffi.cancelGCancellable;

const CPackageDiffEntry = ffi.CPackageDiffEntry;
const CAttributedDiffEntry = ffi.CAttributedDiffEntry;
const DiffStateId = ffi.DiffStateId;

// ── Public imports ───────────────────────────────────────────────────────────
pub const std = @import("std");
pub const ffi = @import("upac-ffi");
pub const c_libs = ffi.c_libs;
pub const data = @import("upac-data");

// ── Errors ───────────────────────────────────────────────────────────────────
pub const DiffError = error{
    RepoOpenFailed,
    CommitNotFound,
    DiffFailed,
    AllocFailed,
    FileNotFound,
    Cancelled,
};

pub const DiffData = struct {
    repo_path: [*:0]const u8,
    root_path: [*:0]const u8,
    prefix_path: [*:0]const u8,

    from_ref: [*:0]const u8,
    to_ref: [*:0]const u8,

    cancel_token: *CancelToken,
};

pub const DiffMachine = struct {
    data: DiffData,
    repo: ?*c_libs.OstreeRepo = null,

    result_packages: ?[]CPackageDiffEntry = null,
    result_files: ?[]CAttributedDiffEntry = null,

    cancellable: ?*c_libs.GCancellable = null,
    gerror: ?*c_libs.GError = null,

    stack: std.ArrayList(DiffStateId),
    allocator: std.mem.Allocator,

    pub fn enter(self: *DiffMachine, state_id: DiffStateId) DiffError!void {
        isBroked(self) catch |err| return err;

        self.stack.append(self.allocator, state_id) catch return DiffError.AllocFailed;
    }

    fn isBroked(self: *DiffMachine) DiffError!void {
        errdefer stateFailed(self);

        if (self.gerror) |err| {
            const is_cancel = err.domain == c_libs.g_io_error_quark() and
                err.code == c_libs.G_IO_ERROR_CANCELLED;

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

    pub inline fn unwrap(self: *DiffMachine, value: anytype, comptime err: DiffError) DiffError!@typeInfo(@TypeOf(value)).optional.child {
        return value orelse {
            stateFailed(self);
            return err;
        };
    }

    pub inline fn check(self: *DiffMachine, value: anytype, comptime err: DiffError) DiffError!@typeInfo(@TypeOf(value)).error_union.payload {
        return value catch {
            stateFailed(self);
            return err;
        };
    }

    pub fn deinit(self: *DiffMachine) void {
        if (self.repo) |repo| c_libs.g_object_unref(repo);
        if (self.gerror) |err| c_libs.g_error_free(err);
        if (self.cancellable) |cancellable| c_libs.g_object_unref(cancellable);

        self.stack.deinit(self.allocator);
    }

    pub fn runPackages(diff_data: DiffData, allocator: std.mem.Allocator) DiffError![]CPackageDiffEntry {
        var machine = DiffMachine{
            .data = diff_data,
            .cancellable = c_libs.g_cancellable_new() orelse return DiffError.Cancelled,
            .stack = std.ArrayList(DiffStateId).empty,
            .allocator = allocator,
        };
        defer machine.deinit();

        diff_data.cancel_token.hook = cancelGCancellable;
        diff_data.cancel_token.hook_ctx = machine.cancellable;
        defer diff_data.cancel_token.reset();

        try machine.enter(.open_repo);
        try states.stateOpenRepo(&machine);
        try machine.enter(.diff_packages);
        try states.stateDiffPackages(&machine);
        try machine.enter(.done);

        return machine.result_packages orelse &.{};
    }

    pub fn runFiles(diff_data: DiffData, allocator: std.mem.Allocator) DiffError![]CAttributedDiffEntry {
        var machine = DiffMachine{
            .data = diff_data,
            .cancellable = c_libs.g_cancellable_new() orelse return DiffError.Cancelled,
            .stack = std.ArrayList(DiffStateId).empty,
            .allocator = allocator,
        };
        defer machine.deinit();

        diff_data.cancel_token.hook = cancelGCancellable;
        diff_data.cancel_token.hook_ctx = machine.cancellable;
        defer diff_data.cancel_token.reset();

        try machine.enter(.open_repo);
        try states.stateOpenRepo(&machine);
        try machine.enter(.diff_files);
        try states.stateDiffFilesAttributed(&machine);
        try machine.enter(.done);

        return machine.result_files orelse &.{};
    }
};
