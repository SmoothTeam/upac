// ── Imports ─────────────────────────────────────────────────────────────────────
const rollback = @import("rollback.zig");
const std = rollback.std;
const c_libs = rollback.c_libs;

const PREFIX = rollback.PREFIX;

const RollbackMachine = rollback.RollbackMachine;
const RollbackError = rollback.RollbackError;
// ── Helpers functions ─────────────────────────────────────────────────────────────────────
pub fn copyFileTo(machine: *RollbackMachine, source_path: [:0]const u8, dest_path: []const u8) !void {
    const content = try std.Io.Dir.cwd().readFileAlloc(machine.io, source_path, machine.allocator, .limited(10 * 1024 * 1024));
    defer machine.allocator.free(content);

    const dest_file = try std.Io.Dir.createFileAbsolute(machine.io, dest_path, .{});
    defer dest_file.close(machine.io);

    var write_buf: [4096]u8 = undefined;
    var bw = dest_file.writer(machine.io, &write_buf);
    const writer = &bw.interface;
    try writer.writeAll(content);
    try writer.flush();
}

pub fn mirrorDir(machine: *RollbackMachine, source_path: [:0]const u8, dest_path: [:0]const u8) !void {
    var source_dir = std.Io.Dir.openDirAbsolute(machine.io, source_path, .{ .iterate = true }) catch return;
    defer source_dir.close(machine.io);

    var walker = try source_dir.walk(machine.allocator);
    defer walker.deinit();

    while (try walker.next(machine.io)) |entry| {
        const dest_child = try std.fs.path.joinZ(machine.allocator, &.{ dest_path, entry.path });
        defer machine.allocator.free(dest_child);

        if (entry.kind == .directory) {
            std.Io.Dir.cwd().createDirPath(machine.io, dest_child) catch {};
            continue;
        }
        if (entry.kind != .file) continue;

        const source_child = try std.fs.path.joinZ(machine.allocator, &.{ source_path, entry.path });
        defer machine.allocator.free(source_child);

        try copyFileTo(machine, source_child, dest_child);
    }
}

pub fn overlayDir(machine: *RollbackMachine, source_path: [:0]const u8, dest_path: [:0]const u8) !void {
    var source_dir = try std.Io.Dir.openDirAbsolute(machine.io, source_path, .{ .iterate = true });
    defer source_dir.close(machine.io);

    var walker = try source_dir.walk(machine.allocator);
    defer walker.deinit();

    while (try walker.next(machine.io)) |entry| {
        const dest_child = try std.fs.path.joinZ(machine.allocator, &.{ dest_path, entry.path });
        defer machine.allocator.free(dest_child);

        if (entry.kind == .directory) {
            std.Io.Dir.cwd().createDirPath(machine.io, dest_child) catch {};
            continue;
        }
        if (entry.kind != .file) continue;

        const source_child = try std.fs.path.joinZ(machine.allocator, &.{ source_path, entry.path });
        defer machine.allocator.free(source_child);

        const conflict = blk: {
            std.Io.Dir.accessAbsolute(machine.io, dest_child, .{}) catch break :blk false;
            break :blk true;
        };

        if (conflict) {
            const dest_conflict = try std.fmt.allocPrint(machine.allocator, "{s}.new", .{dest_child});
            defer machine.allocator.free(dest_conflict);
            try copyFileTo(machine, source_child, dest_conflict);
        } else {
            try copyFileTo(machine, source_child, dest_child);
        }
    }
}


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
