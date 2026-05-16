// ── Imports ─────────────────────────────────────────────────────────────────────
const std = @import("std");

const c_libs = @import("c-libs");

const FileMap = @import("upac-database").FileMap;

const installer = @import("../installer.zig");
const InstallerMachine = installer.InstallerMachine;
const InstallerError = installer.InstallerError;

pub fn collectChecksums(machine: *InstallerMachine, currnet_temp_path: []const u8, file_map: *FileMap) InstallerError!void {
    var dir = std.Io.Dir.openDirAbsolute(machine.io, currnet_temp_path, .{ .iterate = true }) catch return InstallerError.CollectFileChecksumsFailed;
    defer dir.close(machine.io);

    var walker = dir.walk(machine.allocator) catch return InstallerError.CollectFileChecksumsFailed;
    defer walker.deinit();

    while (walker.next(machine.io) catch return InstallerError.CollectFileChecksumsFailed) |entry| {
        var hash_bytes: [std.crypto.hash.sha2.Sha256.digest_length]u8 = undefined;

        if (machine.cancellable) |cancellable| if (c_libs.g_cancellable_is_cancelled(cancellable) != 0) return InstallerError.Cancelled;

        switch (entry.kind) {
            .sym_link => collectSymlinkChecksums(machine, entry, &hash_bytes) catch |err| return err,
            .file => collectFileChecksums(machine, currnet_temp_path, entry, &hash_bytes) catch |err| return err,
            else => continue,
        }

        const hex = std.fmt.bytesToHex(hash_bytes, .lower);
        file_map.put(try machine.allocator.dupe(u8, entry.path), try machine.allocator.dupe(u8, &hex)) catch return InstallerError.CollectFileChecksumsFailed;
    }
}

fn collectSymlinkChecksums(machine: *InstallerMachine, entry: std.Io.Dir.Walker.Entry, hash_out: *[std.crypto.hash.sha2.Sha256.digest_length]u8) !void {
    var link_target_buf: [std.fs.max_path_bytes]u8 = undefined;
    const len = entry.dir.readLink(machine.io, entry.basename, &link_target_buf) catch return InstallerError.CollectFileChecksumsFailed;
    std.crypto.hash.sha2.Sha256.hash(link_target_buf[0..len], hash_out, .{});
}

fn collectFileChecksums(machine: *InstallerMachine, currnet_temp_path: []const u8, entry: std.Io.Dir.Walker.Entry, hash_out: *[std.crypto.hash.sha2.Sha256.digest_length]u8) !void {
    const abs_path = std.fs.path.joinZ(machine.allocator, &.{ currnet_temp_path, entry.path }) catch return InstallerError.CollectFileChecksumsFailed;
    defer machine.allocator.free(abs_path);

    const file = std.Io.Dir.openFileAbsolute(machine.io, abs_path, .{}) catch return InstallerError.CollectFileChecksumsFailed;
    defer file.close(machine.io);

    var file_buf: [4096]u8 = undefined;
    var file_reader = file.reader(machine.io, &file_buf);

    var hash_buf: [4096]u8 = undefined;
    var hasher = file_reader.interface.hashed(std.crypto.hash.sha2.Sha256.init(.{}), &hash_buf);
    _ = hasher.reader.discardRemaining() catch return InstallerError.CollectFileChecksumsFailed;
    hasher.hasher.final(hash_out);
}

pub fn copyFileTo(machine: *InstallerMachine, source_path: [:0]const u8, dest_path: []const u8) InstallerError!void {
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

pub fn copySymlinkTo(machine: *InstallerMachine, source_path: [:0]const u8, dest_path: [:0]const u8) InstallerError!void {
    var link_buf: [std.fs.max_path_bytes]u8 = undefined;
    const len = std.Io.Dir.readLinkAbsolute(machine.io, source_path, &link_buf) catch return InstallerError.WriteFilesFailed;

    const target_c = machine.allocator.dupeZ(u8, link_buf[0..len]) catch return InstallerError.AllocZFailed;
    defer machine.allocator.free(target_c);

    std.Io.Dir.deleteFileAbsolute(machine.io, dest_path) catch {};

    const symlink_create_result = std.os.linux.syscall3(.symlinkat, @intFromPtr(target_c.ptr), @bitCast(@as(isize, std.c.AT.FDCWD)), @intFromPtr(dest_path.ptr));
    if (std.os.linux.errno(symlink_create_result) != .SUCCESS) return InstallerError.WriteFilesFailed;
}
