// ── Imports ─────────────────────────────────────────────────────────────────────
const std = @import("std");

const c_libs = @import("c-libs");

const ffi = @import("upac-ffi");
const InstallProgressFn = ffi.InstallProgressFn;

const CancelToken = ffi.CancelToken;
const cancelGCancellable = ffi.cancelGCancellable;

const types = @import("upac-types");
const Package = types.Package;
const InstallStateId = types.InstallStateId;

const verifying = @import("verifying/verifying.zig");
const preparation = @import("preparation/preparation.zig");
const transaction = @import("transaction/transaction.zig");
const merge = @import("merge/merge.zig");
const checkout = @import("checkout/checkout.zig");
const swap = @import("swap/swap.zig");
// ── Errors ────────────────────────────────────────────────────────────────────
pub const InstallerError = error{
    // Special errors
    AlreadyInstalled,
    PackageNotFound,
    NotEnoughSpace,
    CheckSpaceFailed,
    WriteDatabaseFailed,
    CollectFileChecksumsFailed,
    WriteFilesFailed,
    // Global errors
    PathNotFound,
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

// ── InstallData ───────────────────────────────────────────────────────────────
// A container structure holding all installation parameters: package metadata, paths to the repository and database, as well as retry limits
pub const InstallData = struct {
    packages: []const Package,
    branch: [*:0]const u8,

    repo_path: [*:0]const u8,
    root_path: [*:0]const u8,

    on_progress: ?InstallProgressFn = null,
    progress_ctx: ?*anyopaque = null,
    cancel_token: *CancelToken,
};

// ── InstallerMachine ──────────────────────────────────────────────────────────
// The main structure of a finite-state machine, with information persistence between states
pub const InstallerMachine = struct {
    data: InstallData,

    temp_prefix_path: ?[*:0]u8 = null,
    temp_config_path: ?[*:0]u8 = null,

    cancellable: ?*c_libs.GCancellable = null,
    gerror: ?*c_libs.GError = null,

    allocator: std.mem.Allocator,
    io: std.Io,

    // Reports an installation progress event to the progress callback, if one is set
    pub fn report(self: *InstallerMachine, event: InstallStateId) void {
        const cb = self.data.on_progress orelse return;
        cb(event, self.data.progress_ctx);
    }

    // Correct memory deallocation function
    pub fn deinit(self: *InstallerMachine) void {
        if (self.gerror) |err| c_libs.g_error_free(err);
        if (self.cancellable) |cancellable| c_libs.g_object_unref(cancellable);
    }

    // Initializes the machine, creates the state stack, and launches the first stage—verification
    pub fn run(install_data: InstallData, allocator: std.mem.Allocator) InstallerError!void {
        var state = InstallStateId.verifying;

        var machine = InstallerMachine{
            .data = install_data,

            .cancellable = c_libs.g_cancellable_new() orelse return InstallerError.OutOfMemory,

            .allocator = allocator,
            .io = std.Io.Threaded.global_single_threaded.io(),
        };
        defer machine.deinit();

        install_data.cancel_token.hook = cancelGCancellable;
        install_data.cancel_token.hook_ctx = machine.cancellable;
        defer install_data.cancel_token.reset();

        while (state != .done) {
            machine.report(state);

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
                .done => state = .done,
                .failed => state = .failed,
            }
        }
    }
};
