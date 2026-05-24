// ── Imports ─────────────────────────────────────────────────────────────────────
const std = @import("std");

const c_libs = @import("c-libs");

const installer = @import("../installer.zig");
const InstallerMachine = installer.InstallerMachine;
const InstallerError = installer.InstallerError;

const VerifyingMachine = @import("verifying.zig").VerifyingMachine;

pub fn dirSize(machine: *InstallerMachine, root_path: []const u8) !usize {
    var total_size: usize = 0;

    var dir = std.Io.Dir.openDirAbsolute(machine.io, root_path, .{ .iterate = true }) catch return 0;
    defer dir.close(machine.io);

    var walker = try dir.walk(machine.allocator);
    defer walker.deinit();

    while (try walker.next(machine.io)) |entry| {
        if (entry.kind != .file) continue;
        const file_stats = entry.dir.statFile(machine.io, entry.basename, .{}) catch continue;
        total_size += file_stats.size;
    }

    return total_size;
}
