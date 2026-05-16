const std = @import("std");

const PREFIX = @import("upac-types").PREFIX;

const UninstallerError = @import("../uninstaller.zig").UninstallerError;

pub fn resolveTempDir(root_path: []const u8, allocator: std.mem.Allocator) UninstallerError![:0]u8 {
    var ts: std.os.linux.timespec = undefined;
    _ = std.os.linux.clock_gettime(std.os.linux.CLOCK.REALTIME, &ts);
    const timestamp: i64 = @as(i64, ts.sec) * 1000 + @divTrunc(@as(i64, ts.nsec), 1_000_000);

    var buf: [128]u8 = undefined;
    const name = std.fmt.bufPrint(&buf, "{s}-uninstall-{d}", .{ PREFIX, timestamp }) catch return UninstallerError.AllocZFailed;

    return std.fs.path.joinZ(allocator, &.{ root_path, name }) catch UninstallerError.AllocZFailed;
}
