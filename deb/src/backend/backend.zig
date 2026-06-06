// ── Imports ───────────────────────────────────────────────────────────────────
pub const std = @import("std");

const types = @import("upac-backend-types");
pub const BackendError = types.BackendError;
pub const StateId = types.StateId;
pub const PackageMeta = types.PackageMeta;
pub const PrepareData = types.PrepareData;
pub const PrepareResult = types.PrepareResult;
pub const CancelToken = types.CancelToken;

const verifying = @import("verifying/verifying.zig");
const unpacking = @import("unpacking/unpacking.zig");
const parsing = @import("parsing/parsing.zig");

// ── BackendMachine ────────────────────────────────────────────────────────────
pub const BackendMachine = struct {
    data: PrepareData,

    meta: ?PackageMeta = null,
    temp_package_path: ?[:0]const u8 = null,

    allocator: std.mem.Allocator,
    io: std.Io,

    pub fn deinit(self: *BackendMachine) void {
        if (self.temp_package_path) |path| self.allocator.free(path);
    }

    pub fn hook(self: *BackendMachine, event: StateId) BackendError!void {
        const cb = self.data.on_hook orelse return;
        if (cb(@intFromEnum(event), null, self.data.hook_ctx) == .cancel) return BackendError.Cancelled;
    }

    pub fn run(data: PrepareData, allocator: std.mem.Allocator) BackendError!PrepareResult {
        var machine = BackendMachine{
            .data = data,

            .io = std.Io.Threaded.global_single_threaded.io(),
            .allocator = allocator,
        };
        defer machine.deinit();

        var state = StateId.verifying;
        while (state != .done) {
            machine.hook(state) catch |err| return err;
            switch (state) {
                .verifying => {
                    verifying.run(&machine) catch |err| return err;
                    state = .extracting;
                },
                .extracting => {
                    unpacking.run(&machine) catch |err| return err;
                    state = .reading_meta;
                },
                .reading_meta => {
                    parsing.run(&machine) catch |err| return err;
                    state = .done;
                },
                .done, .special_step => {},
            }
        }

        const temp_package_path = machine.temp_package_path orelse return BackendError.TempDirFailed;
        machine.temp_package_path = null;

        return PrepareResult{
            .meta = machine.meta orelse return BackendError.MetadataNotFound,
            .temp_path = temp_package_path,
        };
    }
};
