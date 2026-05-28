const std = @import("std");

const c_libs = @import("c-libs");

const TransactionMachine = @import("transaction.zig").TransactionMachine;

// ── Helpers ───────────────────────────────────────────────────────────────────
pub fn computeFileChecksum(machine: *TransactionMachine, absolute_file_path: [*:0]const u8) ![32]u8 {
    var file_buf: [4096]u8 = undefined;
    var hash_buf: [4096]u8 = undefined;

    const absolute_file_path_slice = std.mem.span(absolute_file_path);

    const file = std.Io.Dir.openFileAbsolute(machine.files.io, absolute_file_path_slice, .{}) catch return error.DatabaseWriteFailed;
    defer file.close(machine.files.io);

    var file_reader = file.reader(machine.files.io, &file_buf);

    var hasher = file_reader.interface.hashed(std.crypto.hash.sha2.Sha256.init(.{}), &hash_buf);
    _ = hasher.reader.discardRemaining() catch return error.DatabaseWriteFailed;

    var sha256: [32]u8 = undefined;
    hasher.hasher.final(&sha256);
    return sha256;
}

pub fn addToMtree(machine: *TransactionMachine, repo: *c_libs.OstreeRepo, mtree: *c_libs.OstreeMutableTree) !void {
    var timespec: std.os.linux.timespec = undefined;
    _ = std.os.linux.clock_gettime(std.os.linux.CLOCK.REALTIME, &timespec);
    const timestamp: i64 = @as(i64, timespec.sec) * 1000 + @divTrunc(@as(i64, timespec.nsec), 1_000_000);

    const file_path = std.mem.span(machine.files.data.file_paths[machine.current_file_index]);
    const tmp_path = std.mem.span(machine.files.data.tmp_path);

    const path_absolute = if (file_path.len > 0 and file_path[0] == '/') file_path[1..] else file_path;

    const temp_name = try std.fmt.allocPrint(machine.files.allocator, "upac-file-{d}", .{timestamp});
    defer machine.files.allocator.free(temp_name);

    const temp_dir_path = try std.fs.path.joinZ(machine.files.allocator, &.{ tmp_path, temp_name });
    defer machine.files.allocator.free(temp_dir_path);
    defer std.Io.Dir.cwd().deleteTree(machine.files.io, temp_dir_path) catch {};

    const temp_file_path = try std.fs.path.joinZ(machine.files.allocator, &.{ temp_dir_path, path_absolute });
    defer machine.files.allocator.free(temp_file_path);

    // Create parent directory chain inside temp dir
    const parent = std.fs.path.dirname(temp_file_path) orelse return error.RepoTransactionFailed;
    std.Io.Dir.cwd().createDirPath(machine.files.io, parent) catch return error.RepoTransactionFailed;

    std.Io.Dir.copyFileAbsolute(std.mem.span(machine.files.data.file_paths[machine.current_file_index]), temp_file_path, machine.files.io, .{}) catch return error.RepoTransactionFailed;

    const temp_dir_path_c = try machine.files.allocator.dupeZ(u8, temp_dir_path);
    defer machine.files.allocator.free(temp_dir_path_c);

    if (c_libs.ostree_repo_write_dfd_to_mtree(repo, std.c.AT.FDCWD, temp_dir_path_c, mtree, null, machine.files.cancellable, &machine.files.gerror) == 0) return error.RepoTransactionFailed;
}

pub fn removeFromMtree(machine: *TransactionMachine, mtree: *c_libs.OstreeMutableTree) !void {
    const file_path = std.mem.span(machine.files.data.file_paths[machine.current_file_index]);
    const rel_path = if (file_path.len > 0 and file_path[0] == '/') file_path[1..] else file_path;

    var path_components = std.ArrayList([]const u8).empty;
    defer path_components.deinit(machine.files.allocator);

    var iter = std.mem.splitScalar(u8, rel_path, '/');
    while (iter.next()) |part| if (part.len > 0) try path_components.append(machine.files.allocator, part);

    if (path_components.items.len == 0) return error.RepoTransactionFailed;

    var current_subtree: *c_libs.OstreeMutableTree = @ptrCast(@alignCast(c_libs.g_object_ref(mtree)));
    defer c_libs.g_object_unref(current_subtree);

    for (path_components.items[0 .. path_components.items.len - 1]) |dir_part| {
        const dir_part_c = try machine.files.allocator.dupeZ(u8, dir_part);
        defer machine.files.allocator.free(dir_part_c);

        var out_file_checksum: [*c]u8 = null;
        var out_subdir: ?*c_libs.OstreeMutableTree = null;

        if (c_libs.ostree_mutable_tree_lookup(current_subtree, dir_part_c.ptr, &out_file_checksum, &out_subdir, &machine.files.gerror) == 0) {
            if (out_file_checksum != null) c_libs.g_free(out_file_checksum);
            if (machine.files.gerror) |err| {
                c_libs.g_error_free(err);
                machine.files.gerror = null;
            }
            return error.RepoTransactionFailed;
        }

        if (out_file_checksum != null) c_libs.g_free(out_file_checksum);

        const next = out_subdir orelse return error.RepoTransactionFailed;
        c_libs.g_object_unref(current_subtree);
        current_subtree = next;
    }

    const file_name = path_components.items[path_components.items.len - 1];
    const file_name_c = try machine.files.allocator.dupeZ(u8, file_name);
    defer machine.files.allocator.free(file_name_c);

    if (c_libs.ostree_mutable_tree_remove(current_subtree, file_name_c.ptr, 1, &machine.files.gerror) == 0) {
        if (machine.files.gerror) |err| {
            c_libs.g_error_free(err);
            machine.files.gerror = null;
        }
        return error.RepoTransactionFailed;
    }
}
