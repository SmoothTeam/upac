// ── Imports ─────────────────────────────────────────────────────────────────────
const std = @import("std");

const c_libs = @import("c-libs");

const FileMap = @import("upac-database").FileMap;

const types = @import("upac-types");
const PREFIX = types.paths.prefix;
const CONFIG_DIR = types.paths.config_dir;

const installer = @import("../installer.zig");
const InstallerMachine = installer.InstallerMachine;
const InstallerError = installer.InstallerError;

const MergeMachine = @import("merge.zig").MergeMachine;

fn copyFileTo(machine: *InstallerMachine, source_path: [:0]const u8, dest_path: []const u8) InstallerError!void {
    const content = std.Io.Dir.cwd().readFileAlloc(machine.io, source_path, machine.allocator, .limited(10 * 1024 * 1024)) catch return InstallerError.WriteFilesFailed;
    defer machine.allocator.free(content);

    const dest_file = std.Io.Dir.createFileAbsolute(machine.io, dest_path, .{}) catch return InstallerError.WriteFilesFailed;
    defer dest_file.close(machine.io);

    var write_buf: [4096]u8 = undefined;
    var bw = dest_file.writer(machine.io, &write_buf);

    const writer = &bw.interface;
    writer.writeAll(content) catch return InstallerError.WriteFilesFailed;
    writer.flush() catch return InstallerError.WriteFilesFailed;
}

fn copySymlinkTo(machine: *InstallerMachine, source_path: [:0]const u8, dest_path: [:0]const u8) InstallerError!void {
    var link_buf: [std.fs.max_path_bytes]u8 = undefined;
    const len = std.Io.Dir.readLinkAbsolute(machine.io, source_path, &link_buf) catch return InstallerError.WriteFilesFailed;

    const target_c = machine.allocator.dupeZ(u8, link_buf[0..len]) catch return InstallerError.AllocZFailed;
    defer machine.allocator.free(target_c);

    std.Io.Dir.deleteFileAbsolute(machine.io, dest_path) catch {};

    const symlink_create_result = std.os.linux.syscall3(.symlinkat, @intFromPtr(target_c.ptr), @bitCast(@as(isize, std.c.AT.FDCWD)), @intFromPtr(dest_path.ptr));
    if (std.os.linux.errno(symlink_create_result) != .SUCCESS) return InstallerError.WriteFilesFailed;
}

fn fileChecksum(machine: *InstallerMachine, path: [:0]const u8, allocator: std.mem.Allocator) InstallerError![]const u8 {
    const file = std.Io.Dir.openFileAbsolute(machine.io, path, .{}) catch return InstallerError.CollectFileChecksumsFailed;
    defer file.close(machine.io);

    var file_buf: [4096]u8 = undefined;
    var file_reader = file.reader(machine.io, &file_buf);

    var hash_bytes: [std.crypto.hash.sha2.Sha256.digest_length]u8 = undefined;
    var hash_buf: [4096]u8 = undefined;
    var hasher = file_reader.interface.hashed(std.crypto.hash.sha2.Sha256.init(.{}), &hash_buf);
    _ = hasher.reader.discardRemaining() catch return InstallerError.CollectFileChecksumsFailed;
    hasher.hasher.final(&hash_bytes);

    const hex = std.fmt.bytesToHex(hash_bytes, .lower);
    return allocator.dupe(u8, &hex) catch InstallerError.CollectFileChecksumsFailed;
}

pub fn mirrorDir(machine: *InstallerMachine, src: [:0]const u8, dst: [:0]const u8) InstallerError!void {
    var dir = std.Io.Dir.openDirAbsolute(machine.io, src, .{ .iterate = true }) catch return;
    defer dir.close(machine.io);

    var walker = dir.walk(machine.allocator) catch return InstallerError.AllocZFailed;
    defer walker.deinit();

    while (walker.next(machine.io) catch return InstallerError.WriteConfigFailed) |entry| {
        const dest_child = std.fs.path.joinZ(machine.allocator, &.{ dst, entry.path }) catch continue;
        defer machine.allocator.free(dest_child);

        const src_child = std.fs.path.joinZ(machine.allocator, &.{ src, entry.path }) catch continue;
        defer machine.allocator.free(src_child);

        switch (entry.kind) {
            .directory => std.Io.Dir.cwd().createDirPath(machine.io, dest_child) catch {},
            .file => copyFileTo(machine, src_child, dest_child) catch {},
            .sym_link => copySymlinkTo(machine, src_child, dest_child) catch {},
            else => {},
        }
    }
}

pub fn copyEntry(machine: *MergeMachine, kind: std.Io.File.Kind, source: [:0]const u8, dest: [:0]const u8) InstallerError!void {
    if (kind == .sym_link) {
        copySymlinkTo(machine.installer, source, dest) catch return InstallerError.WriteConfigFailed;
    } else {
        copyFileTo(machine.installer, source, dest) catch return InstallerError.WriteConfigFailed;
    }
}

pub fn resolveConflict(machine: *MergeMachine, checksum: ?[32]u8, kind: std.Io.File.Kind, source: [:0]const u8, dest: [:0]const u8) InstallerError!void {
    const user_modified = blk: {
        if (kind == .sym_link) break :blk true;
        const shipped = checksum orelse break :blk true;
        const current_hex = fileChecksum(machine.installer, dest, machine.installer.allocator) catch break :blk true;
        defer machine.installer.allocator.free(current_hex);
        break :blk !std.mem.eql(u8, current_hex, &shipped);
    };

    if (!user_modified) {
        copyFileTo(machine.installer, source, dest) catch return InstallerError.WriteConfigFailed;
        return;
    }

    const dest_new = std.fmt.allocPrintSentinel(machine.installer.allocator, "{s}.new", .{dest}, 0) catch return InstallerError.AllocZFailed;
    defer machine.installer.allocator.free(dest_new);

    std.Io.Dir.deleteFileAbsolute(machine.installer.io, dest_new) catch {};

    if (kind == .sym_link) {
        copySymlinkTo(machine.installer, source, dest_new) catch return InstallerError.WriteConfigFailed;
    } else {
        copyFileTo(machine.installer, source, dest_new) catch return InstallerError.WriteConfigFailed;
    }
}
