const std = @import("std");

const PREFIX = @import("upac-types").paths.prefix;

const rollback = @import("../rollback.zig");
const RollbackError = rollback.RollbackError;

pub fn resolveTempDir(root_path: []const u8, allocator: std.mem.Allocator) RollbackError![:0]u8 {
    var buf: [128]u8 = undefined;
    var timespec: std.os.linux.timespec = undefined;
    _ = std.os.linux.clock_gettime(std.os.linux.CLOCK.REALTIME, &timespec);
    const timestamp: i64 = @as(i64, timespec.sec) * 1000 + @divTrunc(@as(i64, timespec.nsec), 1_000_000);

    const name = std.fmt.bufPrint(&buf, "{s}-rollback-{d}", .{ PREFIX, timestamp }) catch return RollbackError.AllocZFailed;

    return std.fs.path.joinZ(allocator, &.{ root_path, name }) catch RollbackError.AllocZFailed;
}
