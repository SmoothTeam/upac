// ── Imports ─────────────────────────────────────────────────────────────────────
const CSlice = ffi.CSlice;

const CancelToken = ffi.CancelToken;

const RollbackStateId = ffi.RollbackStateId;
const RollbackProgressFn = ffi.RollbackProgressFn;

const cancelGCancellable = ffi.cancelGCancellable;

const states = @import("states.zig");
const stateFailed = states.stateFailed;

const constants = @import("upac-constants");
// ──Public imports ─────────────────────────────────────────────────────────────────────
pub const ffi = @import("upac-ffi");
pub const std = @import("std");
pub const c_libs = ffi.c_libs;

pub const PREFIX = constants.PREFIX;
pub const CONFIG_DIR = constants.CONFIG_DIR;
pub const CONFIG_STAGING_DIR = constants.CONFIG_STAGING_DIR;
// ── Errors ─────────────────────────────────────────────────────────────────────
// Specific rollback errors: failure to open the repository, missing specified commit, or failure to compute the difference between versions
pub const RollbackError = error{
    RepoOpenFailed,
    PathNotFound,
    RepoTransactionFailed,
    CommitNotFound,
    RollbackFailed,
    StagingFailed,
    SwapFailed,
    CleanupFailed,
    AllocZFailed,
    OutOfMemory,
    Cancelled,
    MaxRetriesExceeded,
};

pub const RollbackData = struct {
    repo_path: [*:0]const u8,
    root_path: [*:0]const u8,
    branch: [*:0]const u8,
    commit_hash: [*:0]const u8,

    on_progress: ?RollbackProgressFn = null,
    progress_ctx: ?*anyopaque = null,
    cancel_token: *CancelToken,
};

// ── Rollback ────────────────────────────────────────────────────────────────────
pub const RollbackMachine = struct {
    data: RollbackData,

    repo: ?*c_libs.OstreeRepo = null,
    resolved_checksum: ?[*:0]u8 = null,

    staging_prefix_path_c: ?[:0]const u8 = null,
    staging_config_path_c: ?[:0]const u8 = null,

    cancellable: ?*c_libs.GCancellable = null,
    gerror: ?*c_libs.GError = null,

    allocator: std.mem.Allocator,
    io: std.Io,

    pub fn enter(self: *RollbackMachine, state_id: RollbackStateId) !void {
        isBroked(self) catch |err| return err;

        self.report(state_id);
    }

    fn isBroked(self: *RollbackMachine) RollbackError!void {
        errdefer stateFailed(self);

        if (self.gerror) |err| {
            defer {
                c_libs.g_error_free(err);
                self.gerror = null;
            }

            const is_cancel_error = err.domain == c_libs.g_io_error_quark() and err.code == c_libs.G_IO_ERROR_CANCELLED;

            if (is_cancel_error) {
                if (self.cancellable) |cancellable| c_libs.g_cancellable_cancel(cancellable);
                return RollbackError.Cancelled;
            }
        }

        if (self.cancellable) |cancellable| if (c_libs.g_cancellable_is_cancelled(cancellable) != 0) return RollbackError.Cancelled;
    }

    // Reports an installation progress event to the progress callback, if one is set
    pub fn report(self: *RollbackMachine, event: RollbackStateId) void {
        const cb = self.data.on_progress orelse return;
        cb(event, self.data.progress_ctx);
    }

    pub fn unwrap(self: *RollbackMachine, value: anytype, comptime err: RollbackError) RollbackError!@typeInfo(@TypeOf(value)).optional.child {
        return value orelse {
            stateFailed(self);
            return err;
        };
    }

    pub inline fn check(self: *RollbackMachine, value: anytype, comptime err: RollbackError) RollbackError!@typeInfo(@TypeOf(value)).error_union.payload {
        return value catch {
            stateFailed(self);
            return err;
        };
    }

    pub fn gcheck(self: *RollbackMachine, result: c_int, comptime err: RollbackError) RollbackError!void {
        if (result == 0) {
            stateFailed(self);
            return err;
        }
    }

    pub fn deinit(self: *RollbackMachine) void {
        if (self.repo) |repo| c_libs.g_object_unref(repo);
        if (self.resolved_checksum) |checksum| c_libs.g_free(checksum);

        if (self.staging_prefix_path_c) |path| self.allocator.free(path);
        if (self.staging_config_path_c) |path| self.allocator.free(path);

        if (self.gerror) |err| c_libs.g_error_free(err);
        if (self.cancellable) |cancellable| c_libs.g_object_unref(cancellable);
    }

    pub fn run(rollback_data: RollbackData, allocator: std.mem.Allocator) !void {
        var machine = RollbackMachine{
            .data = rollback_data,

            .cancellable = c_libs.g_cancellable_new() orelse return RollbackError.OutOfMemory,
            .allocator = allocator,
            .io = std.Io.Threaded.global_single_threaded.io(),
        };
        defer machine.deinit();

        rollback_data.cancel_token.hook = cancelGCancellable;
        rollback_data.cancel_token.hook_ctx = machine.cancellable;
        defer rollback_data.cancel_token.reset();

        try states.stateStart(&machine);
    }
};
