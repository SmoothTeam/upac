const std = @import("std");

const types = @import("upac-types");
const PackageMeta = types.PackageMeta;
const Version = types.Version;
const DiffKind = types.DiffKind;
const DiffError = types.DiffError;

const PackageDiffEntry = @import("../packages.zig").PackageDiffEntry;

pub fn appendEntry(entries: *std.ArrayList(PackageDiffEntry), allocator: std.mem.Allocator, meta: PackageMeta, kind: DiffKind) DiffError!void {
    const name_dupe = allocator.dupe(u8, meta.name) catch return DiffError.AllocFailed;
    errdefer allocator.free(name_dupe);

    const parts_dupe = allocator.dupe(u32, meta.version.parts) catch return DiffError.AllocFailed;
    errdefer allocator.free(parts_dupe);

    const pre_dupe: ?[]const u8 = if (meta.version.pre) |pre|
        allocator.dupe(u8, pre) catch return DiffError.AllocFailed
    else
        null;
    errdefer if (pre_dupe) |pre| allocator.free(pre);

    entries.append(allocator, .{
        .name = name_dupe,
        .kind = kind,
        .version = .{
            .epoch = meta.version.epoch,
            .release = meta.version.release,
            .parts = parts_dupe,
            .pre = pre_dupe,
        },
    }) catch return DiffError.AllocFailed;
}

pub fn findInList(list: []const PackageMeta, meta: PackageMeta) ?PackageMeta {
    for (list) |item| {
        if (matchesIdentity(item, meta)) return item;
    }
    return null;
}

pub fn versionEql(a: Version, b: Version) bool {
    if (a.epoch != b.epoch) return false;
    if (a.release != b.release) return false;
    if (a.parts.len != b.parts.len) return false;
    if (!std.mem.eql(u32, a.parts, b.parts)) return false;
    if (a.pre == null and b.pre == null) return true;
    const a_pre = a.pre orelse return false;
    const b_pre = b.pre orelse return false;
    return std.mem.eql(u8, a_pre, b_pre);
}

fn matchesIdentity(a: PackageMeta, b: PackageMeta) bool {
    if (!std.mem.eql(u8, a.name, b.name)) return false;
    if (!std.mem.eql(u8, a.arch, b.arch)) return false;
    if (a.arch_sub == null and b.arch_sub == null) return true;
    const a_sub = a.arch_sub orelse return false;
    const b_sub = b.arch_sub orelse return false;
    return std.mem.eql(u8, a_sub, b_sub);
}
