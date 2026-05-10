// ── Imports ─────────────────────────────────────────────────────────────────────
const rollback = @import("rollback.zig");
const std = rollback.std;
const c_libs = rollback.c_libs;

const PREFIX = rollback.PREFIX;

const RollbackMachine = rollback.RollbackMachine;
const RollbackError = rollback.RollbackError;
// ── Helpers functions ─────────────────────────────────────────────────────────────────────
// Resolve a temporary directory adjacent to root_path (e.g. /usr → /usr-rollback-<timestamp>)
pub fn resolveStagingDir(root_path: []const u8, allocator: std.mem.Allocator) RollbackError![:0]u8 {
    var suffix_buf: [64]u8 = undefined;

    var ts: std.os.linux.timespec = undefined;
    _ = std.os.linux.clock_gettime(std.os.linux.CLOCK.REALTIME, &ts);
    const timestamp: i64 = @as(i64, ts.sec) * 1000 + @divTrunc(@as(i64, ts.nsec), 1_000_000);
    const suffix = std.fmt.bufPrint(&suffix_buf, "{s}-rollback-{d}", .{ PREFIX, timestamp }) catch return error.AllocZFailed;

    return std.fs.path.joinZ(allocator, &.{ root_path, suffix }) catch return error.AllocZFailed;
}
// Resolve a root dir (e.g. /usr → /usr-rollback-<timestamp>)
pub fn resolveRootDir(root_path: []const u8, allocator: std.mem.Allocator) RollbackError![:0]const u8 {
    return std.fs.path.joinZ(allocator, &.{ root_path, PREFIX }) catch return error.AllocZFailed;
}
