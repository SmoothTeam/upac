const std = @import("std");

const types = @import("upac-types");
const DiffEntry = types.DiffEntry;
const FileKind = types.FileKind;

pub fn appendEntry(entries: *std.ArrayList(DiffEntry), allocator: std.mem.Allocator, path: []const u8, kind: FileKind, pkg_name: []const u8, is_user: bool) !void {
    const path_dupe = try allocator.dupe(u8, path);
    errdefer allocator.free(path_dupe);

    const pkg_name_dupe = try allocator.dupe(u8, pkg_name);
    errdefer allocator.free(pkg_name_dupe);

    try entries.append(allocator, .{
        .path = path_dupe,
        .is_user = is_user,
        .kind = kind,
        .package_name = pkg_name_dupe,
    });
}
