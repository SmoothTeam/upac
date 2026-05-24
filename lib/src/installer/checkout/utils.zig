const std = @import("std");

const PREFIX = @import("upac-types").paths.prefix;

const InstallerError = @import("../installer.zig").InstallerError;

pub fn resolveTempDir(root_path: []const u8, allocator: std.mem.Allocator) InstallerError![:0]u8 {
    var buf: [128]u8 = undefined;
    var timespec: std.os.linux.timespec = undefined;
    _ = std.os.linux.clock_gettime(std.os.linux.CLOCK.REALTIME, &timespec);
    const timestamp: i64 = @as(i64, timespec.sec) * 1000 + @divTrunc(@as(i64, timespec.nsec), 1_000_000);

    const name = std.fmt.bufPrint(&buf, "{s}-uninstall-{d}", .{ PREFIX, timestamp }) catch return InstallerError.AllocZFailed;

    return std.fs.path.joinZ(allocator, &.{ root_path, name }) catch InstallerError.AllocZFailed;
}
