const std = @import("std");

const c_libs = @import("c-libs");

const ffi = @import("upac-ffi");
const HookFn = ffi.HookFn;
const CancelToken = ffi.CancelToken;
const cancelGCancellable = ffi.cancelGCancellable;

const types = @import("upac-types");
const DiffKind = types.DiffKind;
const FilesStateId = types.FilesStateId;

const verifying = @import("verifying/verifiyng.zig");
const transaction = @import("transaction/transaction.zig");
const checkout = @import("checkout/checkout.zig");
const swap = @import("swap/swap.zig");
// ── Errors ────────────────────────────────────────────────────────────────────
pub const FilesError = error{
    PathNotFound,
    InvalidFilePath,
    RepoOpenFailed,
    RepoTransactionFailed,
    DatabaseNotFound,
    DatabaseReadFailed,
    DatabaseWriteFailed,
    PackageNotFound,
    CheckoutFailed,
    AllocFailed,
    OutOfMemory,
    AccessDenied,
    Cancelled,
};

// ── FilesData ─────────────────────────────────────────────────────────────────
pub const FilesData = struct {
    file_paths: [][*:0]const u8,
    kind: DiffKind,

    pkg_name: [*:0]const u8,
    pkg_arch: [*:0]const u8,
    pkg_arch_sub: ?[*:0]const u8 = null,

    repo_path: [*:0]const u8,
    root_path: [*:0]const u8,
    tmp_path: [*:0]const u8,
    branch: [*:0]const u8,

    on_hook: ?*const HookFn = null,
    hook_ctx: ?*anyopaque = null,
    cancel_token: *CancelToken,
};

// ── FilesMachine ──────────────────────────────────────────────────────────────
pub const FilesMachine = struct {
    data: FilesData,

    temp_database_path: ?[*:0]u8 = null,
    temp_prefix_path: ?[*:0]u8 = null,

    cancellable: ?*c_libs.GCancellable = null,
    gerror: ?*c_libs.GError = null,

    allocator: std.mem.Allocator,
    io: std.Io,

    pub fn hook(self: *FilesMachine, event: u32, data: ?*const anyopaque) FilesError!void {
        const cb = self.data.on_hook orelse return;
        if (cb(event, data, self.data.hook_ctx) == .cancel) return FilesError.Cancelled;
    }

    pub fn deinit(self: *FilesMachine) void {
        if (self.temp_database_path) |path| {
            const path_slice = std.mem.span(path);
            std.Io.Dir.cwd().deleteTree(self.io, path_slice) catch {};
            self.allocator.free(path_slice);
            self.temp_database_path = null;
        }

        if (self.temp_prefix_path) |path| {
            const path_slice = std.mem.span(path);
            std.Io.Dir.cwd().deleteTree(self.io, path_slice) catch {};
            self.allocator.free(path_slice);
            self.temp_prefix_path = null;
        }

        if (self.gerror) |err| c_libs.g_error_free(err);
        if (self.cancellable) |cancellable| c_libs.g_object_unref(cancellable);
    }

    pub fn run(files_data: FilesData, allocator: std.mem.Allocator) FilesError!void {
        var state = FilesStateId.verifying;

        var machine = FilesMachine{
            .data = files_data,

            .cancellable = c_libs.g_cancellable_new() orelse return FilesError.OutOfMemory,
            .allocator = allocator,
            .io = std.Io.Threaded.global_single_threaded.io(),
        };
        defer machine.deinit();

        files_data.cancel_token.hook = cancelGCancellable;
        files_data.cancel_token.hook_ctx = machine.cancellable;
        defer files_data.cancel_token.reset();

        while (state != .done) {
            try machine.hook(@intFromEnum(state), null);

            if (machine.cancellable) |cancellable| if (c_libs.g_cancellable_is_cancelled(cancellable) != 0) return FilesError.Cancelled;

            switch (state) {
                .verifying => {
                    verifying.run(&machine) catch |err| return err;
                    state = .transaction;
                },
                .transaction => {
                    transaction.run(&machine) catch |err| return err;
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
