const std = @import("std");

const c_libs = @import("c-libs");

const types = @import("upac-types");

pub fn commitDbDirSize(repo: *c_libs.OstreeRepo, checksum: [*c]const u8, cancellable: ?*c_libs.GCancellable, allocator: std.mem.Allocator) usize {
    var total_size: usize = 0;
    var root_gfile: ?*c_libs.GFile = null;
    if (c_libs.ostree_repo_read_commit(repo, checksum, &root_gfile, null, cancellable, null) == 0) return 0;
    defer if (root_gfile) |gfile| c_libs.g_object_unref(gfile);

    const root = root_gfile orelse return 0;

    const database_path = std.fs.path.joinZ(allocator, &.{ types.paths.prefix, types.paths.db_path }) catch return 0;
    defer allocator.free(database_path);

    const database_gfile = c_libs.g_file_resolve_relative_path(root, database_path) orelse return 0;
    defer c_libs.g_object_unref(database_gfile);

    const enumerator = c_libs.g_file_enumerate_children(database_gfile, "standard::size", c_libs.G_FILE_QUERY_INFO_NONE, cancellable, null) orelse return 0;
    defer c_libs.g_object_unref(enumerator);

    while (true) {
        const info = c_libs.g_file_enumerator_next_file(enumerator, cancellable, null) orelse break;
        defer c_libs.g_object_unref(info);

        const file_size = c_libs.g_file_info_get_size(info);
        if (file_size > 0) total_size += @intCast(file_size);
    }

    return total_size;
}
