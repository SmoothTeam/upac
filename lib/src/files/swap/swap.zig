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

    const ref_updated = updateBranchRef(machine);
    if (!ref_updated) _ = atomicSwap(root_prefix.ptr, temp_prefix_path);

    cleanup(machine);
    if (!ref_updated) return FilesError.RepoTransactionFailed;
}

fn updateBranchRef(machine: *FilesMachine) bool {
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
