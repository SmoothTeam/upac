const std = @import("std");

const c_libs = @import("c-libs");

const types = @import("upac-types");
const PREFIX = types.paths.prefix;
const CONFIG_DIR = types.paths.config_dir;

const update = @import("../update.zig");

const UpdateMachine = update.UpdateMachine;
const UpdateError = update.UpdateError;

const RENAME_EXCHANGE: usize = 2;

pub fn run(machine: *UpdateMachine) UpdateError!void {
    if (machine.cancellable) |cancellable| if (c_libs.g_cancellable_is_cancelled(cancellable) != 0) return UpdateError.Cancelled;

    const root_path = std.mem.span(machine.data.root_path);

    const root_prefix = std.fs.path.joinZ(machine.allocator, &.{ root_path, PREFIX }) catch return UpdateError.AllocZFailed;
    defer machine.allocator.free(root_prefix);

    const root_config = std.fs.path.joinZ(machine.allocator, &.{ root_path, CONFIG_DIR }) catch return UpdateError.AllocZFailed;
    defer machine.allocator.free(root_config);

    const temp_prefix_path = machine.temp_prefix_path orelse return UpdateError.CheckoutFailed;
    if (!atomicSwap(temp_prefix_path, root_prefix.ptr)) return UpdateError.RepoTransactionFailed;

    if (machine.temp_config_path) |temp_config_path|
        if (!atomicSwap(temp_config_path, root_config.ptr)) {
            _ = atomicSwap(root_prefix.ptr, temp_prefix_path);
            cleanup(machine);
            return UpdateError.RepoTransactionFailed;
        };

    const ref_updated = updateBranchRef(machine);
    if (!ref_updated) {
        if (machine.temp_config_path) |temp_config_path| _ = atomicSwap(root_config.ptr, temp_config_path);

        _ = atomicSwap(root_prefix.ptr, temp_prefix_path);
    }

    cleanup(machine);
    if (!ref_updated) return UpdateError.RepoTransactionFailed;
}

fn updateBranchRef(machine: *UpdateMachine) bool {
    const gfile = c_libs.g_file_new_for_path(machine.data.repo_path);
    defer c_libs.g_object_unref(gfile);

    const repo = c_libs.ostree_repo_new(gfile);
    defer c_libs.g_object_unref(repo);

    var gerr: ?*c_libs.GError = null;
    if (c_libs.ostree_repo_open(repo, machine.cancellable, &gerr) == 0) {
        if (gerr) |err| c_libs.g_error_free(err);

        return false;
    }

    if (c_libs.ostree_repo_set_ref_immediate(repo, null, machine.data.branch, &machine.new_commit_checksum, machine.cancellable, null) == 0) return false;

    return true;
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

fn cleanup(machine: *UpdateMachine) void {
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
    if (machine.temp_db_path) |path| {
        std.Io.Dir.cwd().deleteTree(machine.io, std.mem.span(path)) catch {};

        machine.allocator.free(std.mem.span(path));
        machine.temp_db_path = null;
    }
}
