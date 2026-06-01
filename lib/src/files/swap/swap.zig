const std = @import("std");

const c_libs = @import("c-libs");

const types = @import("upac-types");
const PREFIX = types.paths.prefix;

const files = @import("../files.zig");
const FilesMachine = files.FilesMachine;
const FilesError = files.FilesError;

const RENAME_EXCHANGE: usize = 2;

pub fn run(machine: *FilesMachine) FilesError!void {
    if (machine.cancellable) |cancellable| if (c_libs.g_cancellable_is_cancelled(cancellable) != 0) return FilesError.Cancelled;

    const root_path = std.mem.span(machine.data.root_path);

    const root_prefix = std.fs.path.joinZ(machine.allocator, &.{ root_path, PREFIX }) catch return FilesError.AllocFailed;
    defer machine.allocator.free(root_prefix);

    const temp_prefix_path = machine.temp_prefix_path orelse return FilesError.CheckoutFailed;

    if (!atomicSwap(temp_prefix_path, root_prefix.ptr)) return FilesError.RepoTransactionFailed;

    cleanup(machine);
}

fn atomicSwap(first_dir: [*:0]const u8, second_dir: [*:0]const u8) bool {
    const result = std.os.linux.syscall5(
        .renameat2,
        @bitCast(@as(isize, std.c.AT.FDCWD)),
        @intFromPtr(first_dir),
        @bitCast(@as(isize, std.c.AT.FDCWD)),
        @intFromPtr(second_dir),
        RENAME_EXCHANGE,
    );
    return std.os.linux.errno(result) == .SUCCESS;
}

fn cleanup(machine: *FilesMachine) void {
    if (machine.temp_prefix_path) |path| {
        const path_slice = std.mem.span(path);
        std.Io.Dir.cwd().deleteTree(machine.io, path_slice) catch {};
        machine.allocator.free(path_slice);
        machine.temp_prefix_path = null;
    }

    if (machine.temp_database_path) |path| {
        const path_slice = std.mem.span(path);
        std.Io.Dir.cwd().deleteTree(machine.io, path_slice) catch {};
        machine.allocator.free(path_slice);
        machine.temp_database_path = null;
    }
}
