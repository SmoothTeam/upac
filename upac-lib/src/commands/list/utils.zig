const list = @import("list.zig");
const std = list.std;
const c_libs = list.c_libs;

const ListMachine = list.ListMachine;
const ListError = list.ListError;

pub fn getRefBody(machine: *ListMachine) ListError!?[]const u8 {
    var checksum: [*c]u8 = null;
    defer if (checksum != null) c_libs.g_free(checksum);

    var commit_variant: ?*c_libs.GVariant = null;
    defer if (commit_variant) |variant| c_libs.g_variant_unref(variant);

    const repo = try machine.unwrap(machine.repo, ListError.RepoOpenFailed);

    if (c_libs.ostree_repo_resolve_rev(repo, machine.data.branch, 1, &checksum, &machine.gerror) == 0 or checksum == null) return ListError.CommitNotFound;

    if (c_libs.ostree_repo_load_variant(repo, c_libs.OSTREE_OBJECT_TYPE_COMMIT, checksum, &commit_variant, &machine.gerror) == 0) return null;

    const body_variant = c_libs.g_variant_get_child_value(commit_variant, 4);
    defer if (body_variant) |variant| c_libs.g_variant_unref(variant);

    var body_len: usize = 0;
    const body_ptr = c_libs.g_variant_get_string(body_variant, &body_len);

    return machine.allocator.dupe(u8, body_ptr[0..body_len]) catch return ListError.AllocFailed;
}

pub fn parsePackageBody(body: []const u8, allocator: std.mem.Allocator) ListError!std.StringHashMap([]const u8) {
    var map = std.StringHashMap([]const u8).init(allocator);
    errdefer freeStringMap(&map, allocator);

    var line_iter = std.mem.splitScalar(u8, body, '\n');
    while (line_iter.next()) |line| {
        const trimmed = std.mem.trim(u8, line, " \t\r");
        if (trimmed.len == 0) continue;

        const sep = std.mem.indexOfScalar(u8, trimmed, ' ') orelse continue;
        const name = trimmed[0..sep];
        const checksum = std.mem.trim(u8, trimmed[sep + 1 ..], " \t");
        if (name.len == 0 or checksum.len == 0) continue;

        const name_dupe = allocator.dupe(u8, name) catch return ListError.AllocFailed;
        const checksum_dupe = allocator.dupe(u8, checksum) catch return ListError.AllocFailed;
        map.put(name_dupe, checksum_dupe) catch return ListError.AllocFailed;
    }
    return map;
}

pub fn freeStringMap(map: *std.StringHashMap([]const u8), allocator: std.mem.Allocator) void {
    var iter = map.iterator();
    while (iter.next()) |entry| {
        allocator.free(entry.key_ptr.*);
        allocator.free(entry.value_ptr.*);
    }
    map.deinit();
}
