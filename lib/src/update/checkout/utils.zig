const std = @import("std");

const PREFIX = @import("upac-types").paths.prefix;

const UpdateError = @import("../update.zig").UpdateError;

pub fn resolveTempDir(root_path: []const u8, allocator: std.mem.Allocator, io: std.Io) UpdateError![:0]u8 {
    var buf: [128]u8 = undefined;
    const timestamp: i64 = @intCast(@divTrunc(std.Io.Clock.real.now(io).nanoseconds, std.time.ns_per_ms));

    const name = std.fmt.bufPrint(&buf, "{s}-update-{d}", .{ PREFIX, timestamp }) catch return UpdateError.AllocZFailed;

    return std.fs.path.joinZ(allocator, &.{ root_path, name }) catch UpdateError.AllocZFailed;
}
