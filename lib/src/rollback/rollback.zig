// ── Imports ─────────────────────────────────────────────────────────────────────
const std = @import("std");

const c_libs = @import("c-libs");

const ffi = @import("upac-ffi");
const RollbackProgressFn = ffi.RollbackProgressFn;

const CancelToken = ffi.CancelToken;
const cancelGCancellable = ffi.cancelGCancellable;

const RollbackStateId = @import("upac-types").RollbackStateId;

const verifying = @import("verifying/verifying.zig");
const merge = @import("merge/merge.zig");
const checkout = @import("checkout/checkout.zig");
const swap = @import("swap/swap.zig");

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
    NotEnoughSpace,
    CheckSpaceFailed,
};

pub const RollbackData = struct {
    repo_path: [*:0]const u8,
    root_path: [*:0]const u8,
    branch: [*:0]const u8,
    commit_hash: [*c]const u8,

    on_progress: ?RollbackProgressFn = null,
    progress_ctx: ?*anyopaque = null,
    cancel_token: *CancelToken,
};

// ── Rollback ────────────────────────────────────────────────────────────────────
pub const RollbackMachine = struct {
    data: RollbackData,

    temp_prefix_path: ?[*:0]u8 = null,
    temp_config_path: ?[*:0]u8 = null,

    cancellable: ?*c_libs.GCancellable = null,
    gerror: ?*c_libs.GError = null,

    allocator: std.mem.Allocator,
    io: std.Io,

    // Reports an installation progress event to the progress callback, if one is set
    pub fn report(self: *RollbackMachine, event: RollbackStateId) void {
        const cb = self.data.on_progress orelse return;
        cb(event, self.data.progress_ctx);
    }

    pub fn deinit(self: *RollbackMachine) void {
        if (self.temp_config_path) |path| self.allocator.free(std.mem.span(path));
        if (self.temp_prefix_path) |path| self.allocator.free(std.mem.span(path));

        if (self.gerror) |err| c_libs.g_error_free(err);
        if (self.cancellable) |cancellable| c_libs.g_object_unref(cancellable);
    }

    pub fn run(rollback_data: RollbackData, allocator: std.mem.Allocator) !void {
        var state = RollbackStateId.verifying;

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

        while (state != .done) {
            machine.report(state);

            switch (state) {
                .verifying => {
                    verifying.run(&machine) catch |err| return err;
                    state = .merge;
                },
                .merge => {
                    merge.run(&machine) catch |err| return err;
                    state = .checkout;
                },
                .checkout => {
                    checkout.run(&machine) catch |err| return err;
                    state = .swap;
                },
                .swap => {
                    swap.run(&machine) catch |err| return err;
                    state = .done;
                },
                .done => state = .done,
                .failed => state = .failed,
            }
        }
    }
};
