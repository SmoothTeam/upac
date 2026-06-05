const std = @import("std");

const PREFIX = @import("upac-types").paths.prefix;

const UninstallerError = @import("../uninstaller.zig").UninstallerError;

pub fn resolveTempDir(root_path: []const u8, allocator: std.mem.Allocator, io: std.Io) UninstallerError![:0]u8 {
    const timestamp: i64 = @intCast(@divTrunc(std.Io.Clock.real.now(io).nanoseconds, std.time.ns_per_ms));

    var buf: [128]u8 = undefined;
    const name = std.fmt.bufPrint(&buf, "{s}-uninstall-{d}", .{ PREFIX, timestamp }) catch return UninstallerError.AllocZFailed;

    return std.fs.path.joinZ(allocator, &.{ root_path, name }) catch UninstallerError.AllocZFailed;
}
