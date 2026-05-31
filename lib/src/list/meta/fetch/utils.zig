const std = @import("std");

const ffi = @import("upac-ffi");
const CSlice = ffi.CSlice;
const CPackageMeta = ffi.CPackageMeta;

const types = @import("upac-types");
const PackageMeta = types.PackageMeta;

const list = @import("../meta.zig");
const ListError = list.ListError;

const required_string_fields = .{ "name", "arch", "maintainer", "description" };
const optional_string_fields = .{ "arch_sub", "license", "url" };

pub fn convertPackageMeta(meta: PackageMeta, allocator: std.mem.Allocator) ListError!CPackageMeta {
    var result: CPackageMeta = .{
        .name = .{ .ptr = null, .len = 0 },
        .arch = .{ .ptr = null, .len = 0 },
        .arch_sub = .{ .ptr = null, .len = 0 },
        .maintainer = .{ .ptr = null, .len = 0 },
        .description = .{ .ptr = null, .len = 0 },
        .license = .{ .ptr = null, .len = 0 },
        .url = .{ .ptr = null, .len = 0 },
        .sha256 = meta.sha256,
        .version = .{
            .epoch = meta.version.epoch,
            .release = meta.version.release,
            .parts = .{ .ptr = undefined, .len = 0 },
            .pre = .{ .ptr = null, .len = 0 },
        },
    };
    errdefer result.free(allocator);

    inline for (required_string_fields) |field_name| {
        const duped = allocator.dupe(u8, @field(meta, field_name)) catch return ListError.AllocFailed;
        @field(result, field_name) = CSlice.fromSlice(duped);
    }

    inline for (optional_string_fields) |field_name| {
        const duped = if (@field(meta, field_name)) |s| allocator.dupe(u8, s) catch return ListError.AllocFailed else null;
        @field(result, field_name) = CSlice.fromSlice(duped);
    }

    const parts = allocator.dupe(u32, meta.version.parts) catch return ListError.AllocFailed;
    result.version.parts = .{ .ptr = parts.ptr, .len = parts.len };

    if (meta.version.pre) |pre| {
        result.version.pre = CSlice.fromSlice(allocator.dupe(u8, pre) catch return ListError.AllocFailed);
    }

    return result;
}
