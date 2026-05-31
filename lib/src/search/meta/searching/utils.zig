const std = @import("std");

const PackageMeta = @import("upac-types").PackageMeta;

const ffi = @import("upac-ffi");
const CPackageMeta = ffi.CPackageMeta;
const CVersion = ffi.CVersion;
const CSlice = ffi.CSlice;

pub fn toCPackageMeta(pkg_meta: PackageMeta) CPackageMeta {
    return .{
        .name = CSlice.fromSlice(pkg_meta.name),
        .version = CVersion{
            .epoch = pkg_meta.version.epoch,
            .release = pkg_meta.version.release,
            .parts = .{ .ptr = @constCast(pkg_meta.version.parts.ptr), .len = pkg_meta.version.parts.len },
            .pre = CSlice.fromSlice(pkg_meta.version.pre),
        },
        .arch = CSlice.fromSlice(pkg_meta.arch),
        .arch_sub = CSlice.fromSlice(pkg_meta.arch_sub),
        .maintainer = CSlice.fromSlice(pkg_meta.maintainer),
        .description = CSlice.fromSlice(pkg_meta.description),
        .license = CSlice.fromSlice(pkg_meta.license),
        .url = CSlice.fromSlice(pkg_meta.url),
        .sha256 = pkg_meta.sha256,
    };
}

pub fn matchesQuery(pkg_meta: *const PackageMeta, query: []const u8) bool {
    if (containsIgnoreCase(pkg_meta.name, query)) return true;

    if (containsIgnoreCase(pkg_meta.arch, query)) return true;

    if (pkg_meta.arch_sub) |arch_sub| if (containsIgnoreCase(arch_sub, query)) return true;

    if (containsIgnoreCase(pkg_meta.maintainer, query)) return true;

    if (containsIgnoreCase(pkg_meta.description, query)) return true;

    if (pkg_meta.license) |license| if (containsIgnoreCase(license, query)) return true;

    if (pkg_meta.url) |url| if (containsIgnoreCase(url, query)) return true;

    return false;
}

fn containsIgnoreCase(haystack: []const u8, needle: []const u8) bool {
    var index: usize = 0;

    if (needle.len == 0 or needle.len > haystack.len) return needle.len == 0;

    while (index <= haystack.len - needle.len) : (index += 1) {
        if (std.ascii.eqlIgnoreCase(haystack[index..][0..needle.len], needle)) return true;
    }

    return false;
}
