const std = @import("std");

const c_libs = @import("c-libs");

const types = @import("upac-types");
const PREFIX = types.PREFIX;
const CONFIG_DIR = types.CONFIG_DIR;

const installer = @import("../installer.zig");

const InstallerMachine = installer.InstallerMachine;
const InstallerError = installer.InstallerError;

const RENAME_EXCHANGE: usize = 2;

pub fn run(machine: *InstallerMachine) InstallerError!void {
    if (machine.cancellable) |cancellable| if (c_libs.g_cancellable_is_cancelled(cancellable) != 0) return InstallerError.Cancelled;

    const root_path = std.mem.span(machine.data.root_path);

    const root_prefix = std.fs.path.joinZ(machine.allocator, &.{ root_path, PREFIX }) catch return InstallerError.AllocZFailed;
    defer machine.allocator.free(root_prefix);

    const root_config = std.fs.path.joinZ(machine.allocator, &.{ root_path, CONFIG_DIR }) catch return InstallerError.AllocZFailed;
    defer machine.allocator.free(root_config);

    const temp_prefix_path = machine.temp_prefix_path orelse return InstallerError.CheckoutFailed;
    if (!atomicSwap(temp_prefix_path, root_prefix.ptr)) return InstallerError.RepoTransactionFailed;

    if (machine.temp_config_path) |temp_config_path|
        if (!atomicSwap(temp_config_path, root_config.ptr)) return InstallerError.RepoTransactionFailed;

    cleanup(machine);
}

fn atomicSwap(first_dir: [*:0]const u8, second_dir: [*:0]const u8) bool {
    const swap_result = std.os.linux.syscall5(
        .renameat2,
        @bitCast(@as(isize, std.c.AT.FDCWD)),
        @intFromPtr(first_dir),
        @bitCast(@as(isize, std.c.AT.FDCWD)),
        @intFromPtr(second_dir),
        RENAME_EXCHANGE,
    );
    return std.os.linux.errno(swap_result) == .SUCCESS;
}

fn cleanup(machine: *InstallerMachine) void {
    if (machine.temp_prefix_path) |path| {
        const snap_temp_prefix_path = std.mem.span(path);

        std.Io.Dir.cwd().deleteTree(machine.io, snap_temp_prefix_path) catch {};

        machine.allocator.free(snap_temp_prefix_path);
        machine.temp_prefix_path = null;
    }
    if (machine.temp_config_path) |path| {
        const snap_temp_config_path = std.mem.span(path);

        std.Io.Dir.cwd().deleteTree(machine.io, snap_temp_config_path) catch {};

        machine.allocator.free(snap_temp_config_path);
        machine.temp_config_path = null;
    }
}
