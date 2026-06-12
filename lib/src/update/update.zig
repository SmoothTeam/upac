// ── Imports ─────────────────────────────────────────────────────────────────────
const std = @import("std");

const c_libs = @import("c-libs");

const ffi = @import("upac-ffi");
const HookFn = ffi.HookFn;

const CancelToken = ffi.CancelToken;
const cancelGCancellable = ffi.cancelGCancellable;

const types = @import("upac-types");
const Package = types.Package;
const UpdateStateId = types.UpdateStateId;

const verifying = @import("verifying/verifying.zig");
const preparation = @import("preparation/preparation.zig");
const transaction = @import("transaction/transaction.zig");
const merge = @import("merge/merge.zig");
const checkout = @import("checkout/checkout.zig");
const swap = @import("swap/swap.zig");
// ── Errors ────────────────────────────────────────────────────────────────────
pub const UpdateError = error{
    // Specific errors
    PackageNotFound,
    NotEnoughSpace,
    CheckSpaceFailed,
    WriteDatabaseFailed,
    ReadDatabaseFailed,
    CollectFileChecksumsFailed,
    WriteFilesFailed,
    AccessDenied,
    // Global errors
    PathNotFound,
    FileNotFound,
    RepoOpenFailed,
    RepoTransactionFailed,
    CheckoutFailed,
    WriteConfigFailed,
    AllocZFailed,
    OutOfMemory,
    CommitNotFound,
    Cancelled,
    MaxRetriesExceeded,
};

// ── UpdateData ────────────────────────────────────────────────────────────────
pub const UpdateData = struct {
    packages: []const Package,
    branch: [*:0]const u8,

    repo_path: [*:0]const u8,
    root_path: [*:0]const u8,
    tmp_path: [*:0]const u8,

    on_hook: ?*const HookFn = null,
    hook_ctx: ?*anyopaque = null,
    cancel_token: *CancelToken,
};

// ── UpdateMachine ─────────────────────────────────────────────────────────────
pub const UpdateMachine = struct {
    data: UpdateData,

    temp_prefix_path: ?[*:0]u8 = null,
    temp_config_path: ?[*:0]u8 = null,
    temp_db_path: ?[*:0]u8 = null,

    new_commit_checksum: [65:0]u8 = std.mem.zeroes([65:0]u8),

    deleted_file_paths: ?[][]const u8 = null,

    cancellable: ?*c_libs.GCancellable = null,
    gerror: ?*c_libs.GError = null,

    allocator: std.mem.Allocator,
    io: std.Io,

    pub fn hook(self: *UpdateMachine, event: u32, data: ?*const anyopaque) UpdateError!void {
        const cb = self.data.on_hook orelse return;
        if (cb(event, data, self.data.hook_ctx) == .cancel) return UpdateError.Cancelled;
    }

    pub fn deinit(self: *UpdateMachine) void {
        if (self.deleted_file_paths) |paths| {
            for (paths) |path| self.allocator.free(path);
            self.allocator.free(paths);
        }
        if (self.gerror) |err| c_libs.g_error_free(err);
        if (self.cancellable) |cancellable| c_libs.g_object_unref(cancellable);
    }

    // Initializes the machine, creates the state stack, and launches the first stage—verification
    pub fn run(update_data: UpdateData, allocator: std.mem.Allocator) UpdateError!void {
        var state = UpdateStateId.verifying;

        var machine = UpdateMachine{
            .data = update_data,

            .cancellable = c_libs.g_cancellable_new() orelse return UpdateError.OutOfMemory,

            .allocator = allocator,
            .io = std.Io.Threaded.global_single_threaded.io(),
        };
        defer machine.deinit();

        update_data.cancel_token.hook = cancelGCancellable;
        update_data.cancel_token.hook_ctx = machine.cancellable;
        defer update_data.cancel_token.reset();

        while (state != .done) {
            try machine.hook(@intFromEnum(state), null);

            switch (state) {
                .verifying => {
                    verifying.run(&machine) catch |err| return err;
                    state = .preparation;
                },
                .preparation => {
                    preparation.run(&machine) catch |err| return err;
                    state = .transaction;
                },
                .transaction => {
                    transaction.run(&machine) catch |err| return err;
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
                .done => {},
            }
        }
    }
};
