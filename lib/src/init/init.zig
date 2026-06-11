const std = @import("std");

const c_libs = @import("c-libs");

const ffi = @import("upac-ffi");
const CancelToken = ffi.CancelToken;
const cancelGCancellable = ffi.cancelGCancellable;
const CRepoMode = ffi.CRepoMode;

const types = @import("upac-types");
const InitStateId = types.InitStateId;

const verifying = @import("verifying/verifying.zig");
const setup = @import("setup/setup.zig");
const commit = @import("commit/commit.zig");
// ── Errors ────────────────────────────────────────────────────────────────────
pub const InitError = error{
    AlreadyInitialized,
    RootNotFound,
    NotADirectory,
    CreateDirFailed,
    OstreeInitFailed,
    OstreeCommitFailed,
    DirectoryNotEmpty,
    SymlinkFailed,
    DatabaseInitFailed,
    AllocFailed,
    OutOfMemory,
    Cancelled,
};

// ── InitData ──────────────────────────────────────────────────────────────────
pub const InitData = struct {
    root_path: [*:0]const u8,
    repo_path: [*:0]const u8,
    repo_mode: CRepoMode,
    branch: [*:0]const u8,
    symlinks: []const [*:0]const u8,
    cancel_token: *CancelToken,
};

// ── InitMachine ───────────────────────────────────────────────────────────────
pub const InitMachine = struct {
    data: InitData,

    cancellable: ?*c_libs.GCancellable = null,
    gerror: ?*c_libs.GError = null,

    allocator: std.mem.Allocator,
    io: std.Io,

    pub fn deinit(self: *InitMachine) void {
        if (self.gerror) |err| c_libs.g_error_free(err);
        if (self.cancellable) |cancellable| c_libs.g_object_unref(cancellable);
    }

    pub fn run(init_data: InitData, allocator: std.mem.Allocator) InitError!void {
        var machine = InitMachine{
            .data = init_data,

            .cancellable = c_libs.g_cancellable_new() orelse return InitError.OutOfMemory,

            .allocator = allocator,
            .io = std.Io.Threaded.global_single_threaded.io(),
        };
        defer machine.deinit();

        init_data.cancel_token.hook = cancelGCancellable;
        init_data.cancel_token.hook_ctx = machine.cancellable;
        defer init_data.cancel_token.reset();

        var state = InitStateId.verifying;
        while (state != .done) {
            if (machine.cancellable) |cancellable| if (c_libs.g_cancellable_is_cancelled(cancellable) != 0) return InitError.Cancelled;

            switch (state) {
                .verifying => {
                    verifying.run(&machine) catch |err| return err;
                    state = .setup;
                },
                .setup => {
                    setup.run(&machine) catch |err| return err;
                    state = .commit;
                },
                .commit => {
                    commit.run(&machine) catch |err| return err;
                    state = .done;
                },
                .done => {},
            }
        }
    }
};
