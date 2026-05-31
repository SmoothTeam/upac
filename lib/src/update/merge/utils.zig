// ── Imports ─────────────────────────────────────────────────────────────────────
const std = @import("std");

const c_libs = @import("c-libs");

const types = @import("upac-types");
const PREFIX = types.paths.prefix;
const CONFIG_DIR = types.paths.config_dir;

const update = @import("../update.zig");
const UpdateMachine = update.UpdateMachine;
const UpdateError = update.UpdateError;

const MergeMachine = @import("merge.zig").MergeMachine;

fn copyFileTo(machine: *UpdateMachine, source_path: [:0]const u8, dest_path: []const u8) UpdateError!void {
    const content = std.Io.Dir.cwd().readFileAlloc(machine.io, source_path, machine.allocator, .limited(10 * 1024 * 1024)) catch return UpdateError.WriteFilesFailed;
    defer machine.allocator.free(content);

    const dest_file = std.Io.Dir.createFileAbsolute(machine.io, dest_path, .{}) catch return UpdateError.WriteFilesFailed;
    defer dest_file.close(machine.io);

    var write_buf: [4096]u8 = undefined;
    var bw = dest_file.writer(machine.io, &write_buf);

    const writer = &bw.interface;
    writer.writeAll(content) catch return UpdateError.WriteFilesFailed;
    writer.flush() catch return UpdateError.WriteFilesFailed;
}

fn copySymlinkTo(machine: *UpdateMachine, source_path: [:0]const u8, dest_path: [:0]const u8) UpdateError!void {
    var link_buf: [std.fs.max_path_bytes]u8 = undefined;
    const len = std.Io.Dir.readLinkAbsolute(machine.io, source_path, &link_buf) catch return UpdateError.WriteFilesFailed;

    const target_c = machine.allocator.dupeZ(u8, link_buf[0..len]) catch return UpdateError.AllocZFailed;
    defer machine.allocator.free(target_c);

    std.Io.Dir.deleteFileAbsolute(machine.io, dest_path) catch {};

    const symlink_create_result = std.os.linux.syscall3(.symlinkat, @intFromPtr(target_c.ptr), @bitCast(@as(isize, std.c.AT.FDCWD)), @intFromPtr(dest_path.ptr));
    if (std.os.linux.errno(symlink_create_result) != .SUCCESS) return UpdateError.WriteFilesFailed;
}

fn fileChecksum(machine: *UpdateMachine, path: [:0]const u8, allocator: std.mem.Allocator) UpdateError![]const u8 {
    const file = std.Io.Dir.openFileAbsolute(machine.io, path, .{}) catch return UpdateError.CollectFileChecksumsFailed;
    defer file.close(machine.io);

    var file_buf: [4096]u8 = undefined;
    var file_reader = file.reader(machine.io, &file_buf);

    var hash_bytes: [std.crypto.hash.sha2.Sha256.digest_length]u8 = undefined;
    var hash_buf: [4096]u8 = undefined;
    var hasher = file_reader.interface.hashed(std.crypto.hash.sha2.Sha256.init(.{}), &hash_buf);
    _ = hasher.reader.discardRemaining() catch return UpdateError.CollectFileChecksumsFailed;
    hasher.hasher.final(&hash_bytes);

    const hex = std.fmt.bytesToHex(hash_bytes, .lower);
    return allocator.dupe(u8, &hex) catch UpdateError.CollectFileChecksumsFailed;
}

pub fn copyEntry(machine: *MergeMachine, kind: std.Io.File.Kind, source: [:0]const u8, dest: [:0]const u8) UpdateError!void {
    if (kind == .sym_link) {
        copySymlinkTo(machine.updater, source, dest) catch return UpdateError.WriteConfigFailed;
    } else {
        copyFileTo(machine.updater, source, dest) catch return UpdateError.WriteConfigFailed;
    }
}

pub fn resolveConflict(machine: *MergeMachine, checksum: ?[32]u8, kind: std.Io.File.Kind, source: [:0]const u8, dest: [:0]const u8) UpdateError!void {
    const user_modified = blk: {
        if (kind == .sym_link) break :blk true;
        const shipped = checksum orelse break :blk true;
        const current_hex = fileChecksum(machine.updater, dest, machine.updater.allocator) catch break :blk true;
        defer machine.updater.allocator.free(current_hex);
        break :blk !std.mem.eql(u8, current_hex, &shipped);
    };

    if (!user_modified) {
        copyFileTo(machine.updater, source, dest) catch return UpdateError.WriteConfigFailed;
        return;
    }

    const dest_new = std.fmt.allocPrintSentinel(machine.updater.allocator, "{s}.new", .{dest}, 0) catch return UpdateError.AllocZFailed;
    defer machine.updater.allocator.free(dest_new);

    std.Io.Dir.deleteFileAbsolute(machine.updater.io, dest_new) catch {};

    if (kind == .sym_link) {
        copySymlinkTo(machine.updater, source, dest_new) catch return UpdateError.WriteConfigFailed;
    } else {
        copyFileTo(machine.updater, source, dest_new) catch return UpdateError.WriteConfigFailed;
    }
}
