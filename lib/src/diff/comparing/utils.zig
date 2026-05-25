const std = @import("std");

const types = @import("upac-types");
const AttributedDiffEntry = types.AttributedDiffEntry;
const DiffKind = types.DiffKind;

pub fn appendEntry(entries: *std.ArrayList(AttributedDiffEntry), allocator: std.mem.Allocator, path: []const u8, kind: DiffKind, pkg_name: []const u8, is_user: bool) !void {
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
