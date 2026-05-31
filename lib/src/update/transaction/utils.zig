const std = @import("std");

const c_libs = @import("c-libs");

const Version = @import("upac-types").Version;

const update = @import("../update.zig");
const UpdateError = update.UpdateError;

const TransactionMachine = @import("transaction.zig").TransactionMachine;

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

        if (c_libs.g_hash_table_size(sub_subdirs) == 0 and c_libs.g_hash_table_size(sub_files) == 0)
            try to_remove.append(allocator, name);
    }

    for (to_remove.items) |name| _ = c_libs.ostree_mutable_tree_remove(tree, name, 1, null);
}

pub fn removeFromMtree(machine: *TransactionMachine, relative_path: []const u8) UpdateError!void {
    var path_components = std.ArrayList([]const u8).empty;
    defer path_components.deinit(machine.updater.allocator);

    const root_mtree = machine.mtree orelse return error.FileNotFound;

    var path_iter = std.mem.splitScalar(u8, relative_path, '/');
    while (path_iter.next()) |part| if (part.len > 0) path_components.append(machine.updater.allocator, part) catch return error.AllocZFailed;

    if (path_components.items.len == 0) return;

    var current_subtree: *c_libs.OstreeMutableTree = @ptrCast(@alignCast(c_libs.g_object_ref(root_mtree)));
    defer c_libs.g_object_unref(current_subtree);

    for (path_components.items[0 .. path_components.items.len - 1]) |dir_component| {
        const dir_component_c = machine.updater.allocator.dupeZ(u8, dir_component) catch return error.AllocZFailed;
        defer machine.updater.allocator.free(dir_component_c);

        var out_file_checksum: [*c]u8 = null;
        var out_subdir: ?*c_libs.OstreeMutableTree = null;

        if (c_libs.ostree_mutable_tree_lookup(current_subtree, dir_component_c.ptr, &out_file_checksum, &out_subdir, &machine.updater.gerror) == 0) {
            if (out_file_checksum != null) c_libs.g_free(out_file_checksum);
            if (machine.updater.gerror) |err| {
                c_libs.g_error_free(err);
                machine.updater.gerror = null;
            }
            return error.FileNotFound;
        }

        if (out_file_checksum != null) c_libs.g_free(out_file_checksum);

        const next = out_subdir orelse return error.FileNotFound;
        c_libs.g_object_unref(current_subtree);
        current_subtree = next;
    }

    const file_name_c = machine.updater.allocator.dupeZ(u8, path_components.items[path_components.items.len - 1]) catch return error.AllocZFailed;
    defer machine.updater.allocator.free(file_name_c);

    if (c_libs.ostree_mutable_tree_remove(current_subtree, file_name_c.ptr, 1, &machine.updater.gerror) == 0) {
        if (machine.updater.gerror) |err| {
            c_libs.g_error_free(err);
            machine.updater.gerror = null;
        }
        return error.FileNotFound;
    }
}

pub fn formatVersion(version: Version, writer: *std.Io.Writer) !void {
    if (version.epoch > 0) try writer.print("{d}:", .{version.epoch});
    for (version.parts, 0..) |part, index| {
        if (index > 0) try writer.print(".", .{});
        try writer.print("{d}", .{part});
    }
    if (version.pre) |pre| try writer.print("~{s}", .{pre});
    if (version.release > 1) try writer.print("-{d}", .{version.release});
}
