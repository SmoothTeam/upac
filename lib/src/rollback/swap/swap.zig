const std = @import("std");

const c_libs = @import("c-libs");

const types = @import("upac-types");
const PREFIX = types.paths.prefix;
const CONFIG_DIR = types.paths.config_dir;

const uninstaller = @import("../rollback.zig");
const RollbackMachine = uninstaller.RollbackMachine;
const RollbackError = uninstaller.RollbackError;

const RENAME_EXCHANGE: usize = 2;

pub fn run(machine: *RollbackMachine) RollbackError!void {
    if (machine.cancellable) |cancellable| if (c_libs.g_cancellable_is_cancelled(cancellable) != 0) return RollbackError.Cancelled;

    const root_path = std.mem.span(machine.data.root_path);

    const root_prefix = std.fs.path.joinZ(machine.allocator, &.{ root_path, PREFIX }) catch return RollbackError.AllocZFailed;
    defer machine.allocator.free(root_prefix);

    const root_config = std.fs.path.joinZ(machine.allocator, &.{ root_path, CONFIG_DIR }) catch return RollbackError.AllocZFailed;
    defer machine.allocator.free(root_config);

    const temp_prefix_path = machine.temp_prefix_path orelse return RollbackError.StagingFailed;
    if (!atomicSwap(temp_prefix_path, root_prefix.ptr)) return RollbackError.RepoTransactionFailed;

    if (machine.temp_config_path) |temp_config_path|
        if (!atomicSwap(temp_config_path, root_config.ptr)) {
            _ = atomicSwap(root_prefix.ptr, temp_prefix_path);
            cleanup(machine);

            return RollbackError.RepoTransactionFailed;
        };

    const ref_updated = updateBranchRef(machine);
    if (!ref_updated) {
        if (machine.temp_config_path) |temp_config_path| _ = atomicSwap(root_config.ptr, temp_config_path);

        _ = atomicSwap(root_prefix.ptr, temp_prefix_path);
    }

    cleanup(machine);
    if (!ref_updated) return RollbackError.RollbackFailed;
}

fn updateBranchRef(machine: *RollbackMachine) bool {
    var pruned_objects: c_libs.gint = 0;
    var pruned_count: c_libs.gint = 0;
    var pruned_size: c_libs.guint64 = 0;
    var gerr: ?*c_libs.GError = null;

    const gfile = c_libs.g_file_new_for_path(machine.data.repo_path);
    defer c_libs.g_object_unref(gfile);

    const repo = c_libs.ostree_repo_new(gfile);
    defer c_libs.g_object_unref(repo);

    if (c_libs.ostree_repo_open(repo, machine.cancellable, &gerr) == 0) {
        if (gerr) |err| c_libs.g_error_free(err);

        return false;
    }

    if (c_libs.ostree_repo_set_ref_immediate(repo, null, machine.data.branch, machine.data.commit_hash, machine.cancellable, null) == 0) return false;

    _ = c_libs.ostree_repo_prune(repo, c_libs.OSTREE_REPO_PRUNE_FLAGS_REFS_ONLY, -1, &pruned_objects, &pruned_count, &pruned_size, machine.cancellable, null);

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

fn cleanup(machine: *RollbackMachine) void {
    if (machine.temp_prefix_path) |path| {
        const temp_prefix_path = std.mem.span(path);

        std.Io.Dir.cwd().deleteTree(machine.io, temp_prefix_path) catch {};

        machine.allocator.free(temp_prefix_path);
        machine.temp_prefix_path = null;
    }

    if (machine.temp_config_path) |path| {
        const temp_config_path = std.mem.span(path);

        std.Io.Dir.cwd().deleteTree(machine.io, temp_config_path) catch {};

        machine.allocator.free(temp_config_path);
        machine.temp_config_path = null;
    }
}
