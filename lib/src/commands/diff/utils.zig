const diff = @import("diff.zig");
const std = diff.std;
const constants = @import("upac-constants");
const c_libs = diff.c_libs;
const data = diff.data;

const DiffMachine = diff.DiffMachine;
const DiffError = diff.DiffError;
const CDiffKind = diff.ffi.CDiffKind;

pub const RawDiffEntry = struct {
    path: []const u8,
    kind: CDiffKind,
};

pub fn getRefBody(machine: *DiffMachine, ref: [*:0]const u8) DiffError!?[]const u8 {
    var commit_checksum: [*c]u8 = null;
    defer if (commit_checksum != null) c_libs.g_free(commit_checksum);

    var commit_variant: ?*c_libs.GVariant = null;
    defer if (commit_variant) |varinat| c_libs.g_variant_unref(varinat);

    const repo = try machine.unwrap(machine.repo, DiffError.RepoOpenFailed);

    if (c_libs.ostree_repo_resolve_rev(repo, ref, 1, &commit_checksum, &machine.gerror) == 0 or commit_checksum == null) return null;

    if (c_libs.ostree_repo_load_variant(repo, c_libs.OSTREE_OBJECT_TYPE_COMMIT, commit_checksum, &commit_variant, &machine.gerror) == 0) return null;

    const body_variant = c_libs.g_variant_get_child_value(commit_variant, 4);
    defer if (body_variant) |variant| c_libs.g_variant_unref(variant);

    var len: usize = 0;
    const ptr = c_libs.g_variant_get_string(body_variant, &len);
    return machine.allocator.dupe(u8, ptr[0..len]) catch return DiffError.AllocFailed;
}

pub fn parsePackageBody(body: []const u8, allocator: std.mem.Allocator) DiffError!std.StringHashMap([]const u8) {
    var map = std.StringHashMap([]const u8).init(allocator);
    errdefer freeStringMap(&map, allocator);

    var iter = std.mem.splitScalar(u8, body, '\n');
    while (iter.next()) |line| {
        const trimmed_line = std.mem.trim(u8, line, " \t\r");
        if (trimmed_line.len == 0) continue;
        const separator_index = std.mem.indexOfScalar(u8, trimmed_line, ' ') orelse continue;
        const name = trimmed_line[0..separator_index];
        const checksum = std.mem.trim(u8, trimmed_line[separator_index + 1 ..], " \t");
        if (name.len == 0 or checksum.len == 0) continue;
        const key_dupe = allocator.dupe(u8, name) catch return DiffError.AllocFailed;
        const value_dupe = allocator.dupe(u8, checksum) catch return DiffError.AllocFailed;
        map.put(key_dupe, value_dupe) catch return DiffError.AllocFailed;
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

pub fn resolveCommitRoot(machine: *DiffMachine, ref: [*:0]const u8) DiffError!*c_libs.GFile {
    var commit_checksum: [*c]u8 = null;
    defer if (commit_checksum != null) c_libs.g_free(commit_checksum);

    const repo = try machine.unwrap(machine.repo, DiffError.RepoOpenFailed);

    if (c_libs.ostree_repo_resolve_rev(repo, ref, 0, &commit_checksum, &machine.gerror) == 0 or commit_checksum == null)
        return DiffError.CommitNotFound;

    var root_gfile: ?*c_libs.GFile = null;
    if (c_libs.ostree_repo_read_commit(repo, commit_checksum, &root_gfile, null, machine.cancellable, &machine.gerror) == 0)
        return DiffError.CommitNotFound;

    return root_gfile orelse DiffError.CommitNotFound;
}

pub fn buildFilePkgMap(machine: *DiffMachine, ref: [*:0]const u8, out: *std.StringHashMap([]const u8)) DiffError!void {
    const body = (try getRefBody(machine, ref)) orelse return;
    defer machine.allocator.free(body);

    var pkg_map = try parsePackageBody(body, machine.allocator);
    defer freeStringMap(&pkg_map, machine.allocator);

    var iter = pkg_map.iterator();
    while (iter.next()) |entry| {
        const abs_db_path = std.fs.path.join(machine.allocator, &.{ std.mem.span(machine.data.root_path), constants.DB_RELATIVE_PATH }) catch continue;
        defer machine.allocator.free(abs_db_path);
        var file_map = data.readFiles(abs_db_path, entry.value_ptr.*, machine.allocator) catch continue;
        defer data.freeFileMap(&file_map, machine.allocator);

        var file_iter = file_map.iterator();
        while (file_iter.next()) |fe| {
            if (out.contains(fe.key_ptr.*)) continue;
            const key_dupe = machine.allocator.dupe(u8, fe.key_ptr.*) catch return DiffError.AllocFailed;
            const value_dupe = machine.allocator.dupe(u8, entry.key_ptr.*) catch return DiffError.AllocFailed;
            out.put(key_dupe, value_dupe) catch return DiffError.AllocFailed;
        }
    }
}

pub fn collectEntries(arr: *c_libs.GPtrArray, root_gfile: *c_libs.GFile, kind: CDiffKind, use_target: bool, result: *std.ArrayList(RawDiffEntry), allocator: std.mem.Allocator) DiffError!void {
    var index: usize = 0;
    while (index < arr.*.len) : (index += 1) {
        const item_gfile: *c_libs.GFile = if (use_target) blk: {
            const diff_item: *c_libs.OstreeDiffItem = @ptrCast(@alignCast(arr.*.pdata[index]));
            break :blk @ptrCast(diff_item.target);
        } else blk: {
            break :blk @ptrCast(@alignCast(arr.*.pdata[index]));
        };

        const relative_path = c_libs.g_file_get_relative_path(@ptrCast(root_gfile), item_gfile);
        defer if (relative_path != null) c_libs.g_free(@ptrCast(relative_path));
        if (relative_path == null) continue;

        const path_dupe = allocator.dupe(u8, std.mem.span(relative_path)) catch return DiffError.AllocFailed;
        result.append(allocator, .{ .path = path_dupe, .kind = kind }) catch return DiffError.AllocFailed;
    }
}
