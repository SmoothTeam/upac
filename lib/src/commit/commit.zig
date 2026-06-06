// ── Imports ─────────────────────────────────────────────────────────────────────
const std = @import("std");

const c_libs = @import("c-libs");

const ffi = @import("upac-ffi");
const HookFn = ffi.HookFn;

const CancelToken = ffi.CancelToken;
const cancelGCancellable = ffi.cancelGCancellable;

const CommitStateId = @import("upac-types").CommitStateId;

const verifying = @import("verifying/verifying.zig");
const transaction = @import("transaction/transaction.zig");

// ── Errors ────────────────────────────────────────────────────────────────────
pub const CommitError = error{
    RepoOpenFailed,
    RepoTransactionFailed,
    CommitFailed,
    PathNotFound,
    AllocZFailed,
    OutOfMemory,
    Cancelled,
};

// ── CommitData ────────────────────────────────────────────────────────────────
pub const CommitData = struct {
    repo_path: [*:0]const u8,
    root_path: [*:0]const u8,
    branch: [*:0]const u8,
    message: [*:0]const u8,

    on_hook: ?*const HookFn = null,
    hook_ctx: ?*anyopaque = null,

    cancel_token: *CancelToken,
};

// ── CommitMachine ─────────────────────────────────────────────────────────────
pub const CommitMachine = struct {
    data: CommitData,

    cancellable: ?*c_libs.GCancellable = null,
    gerror: ?*c_libs.GError = null,

    allocator: std.mem.Allocator,
    io: std.Io,

    pub fn hook(self: *CommitMachine, event: u32, data: ?*const anyopaque) CommitError!void {
        const cb = self.data.on_hook orelse return;
        if (cb(event, data, self.data.hook_ctx) == .cancel) return CommitError.Cancelled;
    }

    pub fn deinit(self: *CommitMachine) void {
        if (self.gerror) |err| c_libs.g_error_free(err);
        if (self.cancellable) |cancellable| c_libs.g_object_unref(cancellable);
    }

    pub fn run(commit_data: CommitData, allocator: std.mem.Allocator) CommitError!void {
        var state = CommitStateId.verifying;

        var machine = CommitMachine{
            .data = commit_data,

            .cancellable = c_libs.g_cancellable_new() orelse return CommitError.OutOfMemory,

            .allocator = allocator,
            .io = std.Io.Threaded.global_single_threaded.io(),
        };
        defer machine.deinit();

        commit_data.cancel_token.hook = cancelGCancellable;
        commit_data.cancel_token.hook_ctx = machine.cancellable;
        defer commit_data.cancel_token.reset();

        while (state != .done) {
            try machine.hook(@intFromEnum(state), null);

            switch (state) {
                .verifying => {
                    verifying.run(&machine) catch |err| return err;
                    state = .transaction;
                },
                .transaction => {
                    transaction.run(&machine) catch |err| return err;
                    state = .done;
                },
                .done => {},
            }
        }
    }
};
