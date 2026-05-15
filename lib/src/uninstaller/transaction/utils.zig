const std = @import("std");

const uninstaller = @import("../uninstaller.zig");
const c_libs = uninstaller.ffi.c_libs;

const DB_RELATIVE_PATH = uninstaller.types.DB_RELATIVE_PATH;

const UninstallerMachine = uninstaller.UninstallerMachine;
const UninstallerError = uninstaller.UninstallerError;

const TransactionMachine = @import("transaction.zig").TransactionMachine;

// ── Helpers ───────────────────────────────────────────────────────────────────
pub fn loadCommitBody(machine: *TransactionMachine, checksum: [*c]const u8) UninstallerError![]const u8 {
    var commit_variant: ?*c_libs.GVariant = null;
    defer if (commit_variant) |variant| c_libs.g_variant_unref(variant);

    const repo = machine.repo orelse return UninstallerError.RepoOpenFailed;

    if (c_libs.ostree_repo_load_variant(repo, c_libs.OSTREE_OBJECT_TYPE_COMMIT, checksum, &commit_variant, &machine.uninstaller.gerror) == 0) return UninstallerError.RepoTransactionFailed;

    const body_variant = c_libs.g_variant_get_child_value(commit_variant, 4);
    defer if (body_variant) |variant| c_libs.g_variant_unref(variant);

    var body_len: usize = 0;
    const body_ptr = c_libs.g_variant_get_string(body_variant, &body_len);
    if (body_len == 0) return UninstallerError.CommitNotFound;

    return machine.uninstaller.allocator.dupe(u8, body_ptr[0..body_len]) catch UninstallerError.AllocZFailed;
}

pub fn removeDbEntry(machine: *TransactionMachine, checksum: []const u8, comptime ext: []const u8) void {
    var buf: [300]u8 = undefined;
    const filename = std.fmt.bufPrint(&buf, "{s}" ++ ext, .{checksum}) catch return;
    const path = std.fs.path.join(machine.uninstaller.allocator, &.{ DB_RELATIVE_PATH, filename }) catch return;
    defer machine.uninstaller.allocator.free(path);
    removeFromMtree(machine, path) catch {};
}

pub fn removeEmptyDirs(tree: *c_libs.OstreeMutableTree, allocator: std.mem.Allocator) std.mem.Allocator.Error!void {
    const subdirs = c_libs.ostree_mutable_tree_get_subdirs(tree);

    var to_remove = std.ArrayList([*:0]const u8).empty;
    defer to_remove.deinit(allocator);

    var iter: c_libs.GHashTableIter = undefined;
    c_libs.g_hash_table_iter_init(&iter, subdirs);

    var key_ptr: ?*anyopaque = null;
    var val_ptr: ?*anyopaque = null;

    while (c_libs.g_hash_table_iter_next(&iter, &key_ptr, &val_ptr) != 0) {
        const name: [*:0]const u8 = @ptrCast(key_ptr.?);
        const subdir: *c_libs.OstreeMutableTree = @ptrCast(@alignCast(val_ptr.?));

        try removeEmptyDirs(subdir, allocator);

        const sub_subdirs = c_libs.ostree_mutable_tree_get_subdirs(subdir);
        const sub_files = c_libs.ostree_mutable_tree_get_files(subdir);

        if (c_libs.g_hash_table_size(sub_subdirs) == 0 and c_libs.g_hash_table_size(sub_files) == 0) try to_remove.append(allocator, name);
    }

    for (to_remove.items) |name| _ = c_libs.ostree_mutable_tree_remove(tree, name, 1, null);
}

pub fn removeFromMtree(machine: *TransactionMachine, relative_path: []const u8) UninstallerError!void {
    var path_components = std.ArrayList([]const u8).empty;
    defer path_components.deinit(machine.uninstaller.allocator);

    const root_mtree = machine.mtree orelse return error.FileNotFound;

    var path_components_iter = std.mem.splitScalar(u8, relative_path, '/');
    while (path_components_iter.next()) |path_part| if (path_part.len > 0) path_components.append(machine.uninstaller.allocator, path_part) catch return error.AllocZFailed;

    if (path_components.items.len == 0) return;

    var current_subtree: *c_libs.OstreeMutableTree = @ptrCast(@alignCast(c_libs.g_object_ref(root_mtree)));
    defer c_libs.g_object_unref(current_subtree);

    for (path_components.items[0 .. path_components.items.len - 1]) |directory_component| {
        const directory_component_c = machine.uninstaller.allocator.dupeZ(u8, directory_component) catch return error.AllocZFailed;
        defer machine.uninstaller.allocator.free(directory_component_c);

        var out_file_checksum: [*c]u8 = null;
        var out_subdir: ?*c_libs.OstreeMutableTree = null;

        if (c_libs.ostree_mutable_tree_lookup(current_subtree, directory_component_c.ptr, &out_file_checksum, &out_subdir, &machine.uninstaller.gerror) == 0) {
            if (out_file_checksum != null) c_libs.g_free(out_file_checksum);
            if (machine.uninstaller.gerror) |err| {
                c_libs.g_error_free(err);
                machine.uninstaller.gerror = null;
            }
            return error.FileNotFound;
        }

        if (out_file_checksum != null) c_libs.g_free(out_file_checksum);

        const next = out_subdir orelse return error.FileNotFound;
        c_libs.g_object_unref(current_subtree);
        current_subtree = next;
    }

    const file_name_c = machine.uninstaller.allocator.dupeZ(u8, path_components.items[path_components.items.len - 1]) catch return error.AllocZFailed;
    defer machine.uninstaller.allocator.free(file_name_c);

    if (c_libs.ostree_mutable_tree_remove(current_subtree, file_name_c.ptr, 1, &machine.uninstaller.gerror) == 0) {
        if (machine.uninstaller.gerror) |err| {
            c_libs.g_error_free(err);
            machine.uninstaller.gerror = null;
        }
        return error.FileNotFound;
    }
}
