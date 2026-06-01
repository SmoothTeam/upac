// ── Imports ─────────────────────────────────────────────────────────────────────
const std = @import("std");

const c_libs = @import("c-libs");

const types = @import("upac-types");
const UninstallPackage = types.UninstallPackage;
const UninstallStateId = types.UninstallStateId;

const ffi = @import("upac-ffi");
const HookFn = ffi.HookFn;

const CancelToken = ffi.CancelToken;
const cancelGCancellable = ffi.cancelGCancellable;

const verifying = @import("verifying/verifying.zig");
const transaction = @import("transaction/transaction.zig");
const merge = @import("merge/merge.zig");
const checkout = @import("checkout/checkout.zig");
const swap = @import("swap/swap.zig");
// ── Errors ─────────────────────────────────────────────────────────────────────
// Errors specific to the removal process
pub const UninstallerError = error{
    // Specific errors
    PackageNotFound,
    FileNotFound,
    FileMapCorrupted,
    CommitNotFound,
    StagingNotCleaned,
    ReadDatabaseFailed,
    // Global errors
    PathNotFound,
    RepoOpenFailed,
    RepoTransactionFailed,
    CheckoutFailed,
    AllocZFailed,
    OutOfMemory,
    Cancelled,
    MaxRetriesExceeded,
};

// ── UninstallerFSM data ─────────────────────────────────────────────────────────────────────
// Set of input parameters: package name, paths to the repository and database, as well as the target branch for the commit
pub const UninstallData = struct {
    packages: []const UninstallPackage,
    branch: [*:0]const u8,

    repo_path: [*:0]const u8,
    root_path: [*:0]const u8,

    on_hook: ?*const HookFn = null,
    hook_ctx: ?*anyopaque = null,
    cancel_token: *CancelToken,
};

// ── UninstallerFSM ─────────────────────────────────────────────────────────────────────
// Uninstaller state container for fsm data between states
pub const UninstallerMachine = struct {
    data: UninstallData,

    temp_prefix_path: ?[*:0]u8 = null,
    temp_config_path: ?[*:0]u8 = null,

    cancellable: ?*c_libs.GCancellable = null,
    gerror: ?*c_libs.GError = null,

    allocator: std.mem.Allocator,
    io: std.Io,

    pub fn hook(self: *UninstallerMachine, event: u32, data: ?*const anyopaque) UninstallerError!void {
        const cb = self.data.on_hook orelse return;
        if (cb(event, data, self.data.hook_ctx) == .cancel) return UninstallerError.Cancelled;
    }

    // Releases all resources: native Zig memory, the file hash map, and OSTree system C objects
    pub fn deinit(self: *UninstallerMachine) void {
        if (self.temp_prefix_path) |path| c_libs.g_free(path);
        if (self.temp_config_path) |path| c_libs.g_free(path);

        if (self.gerror) |err| c_libs.g_error_free(err);
        if (self.cancellable) |cancellable| c_libs.g_object_unref(cancellable);
    }

    // Entry point: initializes the uninstallation engine and launches the package removal process
    pub fn run(uninstall_data: UninstallData, allocator: std.mem.Allocator) !void {
        var state = UninstallStateId.verifying;

        var machine = UninstallerMachine{
            .data = uninstall_data,

            .cancellable = c_libs.g_cancellable_new() orelse return error.OutOfMemory,

            .allocator = allocator,
            .io = std.Io.Threaded.global_single_threaded.io(),
        };
        defer machine.deinit();

        uninstall_data.cancel_token.hook = cancelGCancellable;
        uninstall_data.cancel_token.hook_ctx = machine.cancellable;
        defer uninstall_data.cancel_token.reset();

        while (state != .done) {
            try machine.hook(@intFromEnum(state), null);

            switch (state) {
                .verifying => {
                    verifying.run(&machine) catch |err| return err;
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
