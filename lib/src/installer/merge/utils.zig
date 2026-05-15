// ── Imports ─────────────────────────────────────────────────────────────────────
const std = @import("std");

const installer = @import("../installer.zig");
const c_libs = installer.ffi.c_libs;

const FileMap = installer.database.FileMap;

const InstallerMachine = installer.InstallerMachine;
const InstallerError = installer.InstallerError;

const PREFIX = installer.types.PREFIX;
const CONFIG_DIR = installer.types.CONFIG_DIR;

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

pub fn copyEntry(machine: *MergeMachine, kind: std.Io.File.Kind, source: [:0]const u8, dest: [:0]const u8) InstallerError!void {
    if (kind == .sym_link) {
        copySymlinkTo(machine.installer, source, dest) catch return InstallerError.WriteConfigFailed;
    } else {
        copyFileTo(machine.installer, source, dest) catch return InstallerError.WriteConfigFailed;
    }
}

pub fn resolveConflict(machine: *MergeMachine, db: FileMap, kind: std.Io.File.Kind, source: [:0]const u8, dest: [:0]const u8, entry_path: []const u8) InstallerError!void {
    const db_key = std.fs.path.join(machine.installer.allocator, &.{ PREFIX, CONFIG_DIR, entry_path }) catch return InstallerError.AllocZFailed;
    defer machine.installer.allocator.free(db_key);

    const shipped_checksum = db.get(db_key);

    const user_modified = blk: {
        if (kind == .sym_link) break :blk true;
        const shipped = shipped_checksum orelse break :blk true;
        const current_hex = fileChecksum(machine.installer, dest, machine.installer.allocator) catch break :blk true;
        defer machine.installer.allocator.free(current_hex);
        break :blk !std.mem.eql(u8, current_hex, shipped);
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
