const std = @import("std");

const c_libs = @import("c-libs");

const list = @import("list.zig");
const ListMachine = list.ListMachine;
const ListError = list.ListError;

pub fn getRefBody(machine: *ListMachine) ListError!?[]const u8 {
    var checksum: [*c]u8 = null;
    defer if (checksum != null) c_libs.g_free(checksum);

    var commit_variant: ?*c_libs.GVariant = null;
    defer if (commit_variant) |variant| c_libs.g_variant_unref(variant);

    const repo = machine.repo orelse return ListError.RepoOpenFailed;

    if (c_libs.ostree_repo_resolve_rev(repo, machine.data.branch, 1, &checksum, &machine.gerror) == 0 or checksum == null) return ListError.CommitNotFound;

    if (c_libs.ostree_repo_load_variant(repo, c_libs.OSTREE_OBJECT_TYPE_COMMIT, checksum, &commit_variant, &machine.gerror) == 0) return null;

    const body_variant = c_libs.g_variant_get_child_value(commit_variant, 4);
    defer if (body_variant) |variant| c_libs.g_variant_unref(variant);

    var body_len: usize = 0;
    const body_ptr = c_libs.g_variant_get_string(body_variant, &body_len);

    return machine.allocator.dupe(u8, body_ptr[0..body_len]) catch |err| return err;
}

pub fn parsePackageBody(package_body: []const u8, allocator: std.mem.Allocator) ListError!std.StringHashMap([]const u8) {
    var package_map = std.StringHashMap([]const u8).init(allocator);
    errdefer freeStringMap(&package_map, allocator);

    var package_body_iter = std.mem.splitScalar(u8, package_body, '\n');
    while (package_body_iter.next()) |line| {
        const trimmed_line = std.mem.trim(u8, line, " \t\r");
        if (trimmed_line.len == 0) continue;

        const separator_index = std.mem.indexOfScalar(u8, trimmed_line, ' ') orelse continue;

        const name = trimmed_line[0..separator_index];
        const checksum = std.mem.trim(u8, trimmed_line[separator_index + 1 ..], " \t");
        if (name.len == 0 or checksum.len == 0) continue;

        const name_dupe = allocator.dupe(u8, name) catch |err| return err;
        const checksum_dupe = allocator.dupe(u8, checksum) catch |err| return err;
        package_map.put(name_dupe, checksum_dupe) catch |err| return err;
    }
    return package_map;
}

pub fn freeStringMap(string_map: *std.StringHashMap([]const u8), allocator: std.mem.Allocator) void {
    var string_map_iter = string_map.iterator();
    while (string_map_iter.next()) |entry| {
        allocator.free(entry.key_ptr.*);
        allocator.free(entry.value_ptr.*);
    }
    string_map.deinit();
}
