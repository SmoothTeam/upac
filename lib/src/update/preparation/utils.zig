const std = @import("std");

const c_libs = @import("c-libs");

const types = @import("upac-types");
const FileEntry = types.FileEntry;

const update = @import("../update.zig");
const UpdateMachine = update.UpdateMachine;
const UpdateError = update.UpdateError;

pub fn collectChecksums(machine: *UpdateMachine, current_temp_path: []const u8, file_entries: *std.ArrayList(FileEntry)) UpdateError!void {
    var dir = std.Io.Dir.openDirAbsolute(machine.io, current_temp_path, .{ .iterate = true }) catch return UpdateError.CollectFileChecksumsFailed;
    defer dir.close(machine.io);

    var walker = dir.walk(machine.allocator) catch return UpdateError.CollectFileChecksumsFailed;
    defer walker.deinit();

    while (walker.next(machine.io) catch return UpdateError.CollectFileChecksumsFailed) |entry| {
        var hash_bytes: [std.crypto.hash.sha2.Sha256.digest_length]u8 = undefined;

        if (machine.cancellable) |cancellable| if (c_libs.g_cancellable_is_cancelled(cancellable) != 0) return UpdateError.Cancelled;

        switch (entry.kind) {
            .sym_link => collectSymlinkChecksum(entry, &hash_bytes) catch return UpdateError.CollectFileChecksumsFailed,
            .file => collectFileChecksum(machine, current_temp_path, entry, &hash_bytes) catch return UpdateError.CollectFileChecksumsFailed,
            else => continue,
        }

        const path = machine.allocator.dupe(u8, entry.path) catch return UpdateError.CollectFileChecksumsFailed;
        file_entries.append(machine.allocator, .{ .path = path, .sha256 = hash_bytes, .is_user = false }) catch {
            machine.allocator.free(path);
            return UpdateError.CollectFileChecksumsFailed;
        };
    }
}

fn collectSymlinkChecksum(entry: std.Io.Dir.Walker.Entry, hash_out: *[std.crypto.hash.sha2.Sha256.digest_length]u8) !void {
    var link_target_buf: [std.fs.max_path_bytes]u8 = undefined;

    const len = entry.dir.readLink(undefined, entry.basename, &link_target_buf) catch return error.CollectFileChecksumsFailed;

    std.crypto.hash.sha2.Sha256.hash(link_target_buf[0..len], hash_out, .{});
}

fn collectFileChecksum(machine: *UpdateMachine, current_temp_path: []const u8, entry: std.Io.Dir.Walker.Entry, hash_out: *[std.crypto.hash.sha2.Sha256.digest_length]u8) !void {
    var file_buf: [4096]u8 = undefined;
    var hash_buf: [4096]u8 = undefined;

    const absolute_path = std.fs.path.joinZ(machine.allocator, &.{ current_temp_path, entry.path }) catch return error.CollectFileChecksumsFailed;
    defer machine.allocator.free(absolute_path);

    const file = std.Io.Dir.openFileAbsolute(machine.io, absolute_path, .{}) catch return error.CollectFileChecksumsFailed;
    defer file.close(machine.io);

    var file_reader = file.reader(machine.io, &file_buf);

    var hasher = file_reader.interface.hashed(std.crypto.hash.sha2.Sha256.init(.{}), &hash_buf);
    _ = hasher.reader.discardRemaining() catch return error.CollectFileChecksumsFailed;
    hasher.hasher.final(hash_out);
}

pub fn copyFileTo(machine: *UpdateMachine, source_path: [:0]const u8, dest_path: []const u8) UpdateError!void {
    var write_buf: [4096]u8 = undefined;

    const content = std.Io.Dir.cwd().readFileAlloc(machine.io, source_path, machine.allocator, .limited(10 * 1024 * 1024)) catch return UpdateError.WriteFilesFailed;
    defer machine.allocator.free(content);

    const destination_file = std.Io.Dir.createFileAbsolute(machine.io, dest_path, .{}) catch return UpdateError.WriteFilesFailed;
    defer destination_file.close(machine.io);

    var buffer_writer = destination_file.writer(machine.io, &write_buf);

    const writer = &buffer_writer.interface;
    writer.writeAll(content) catch return UpdateError.WriteFilesFailed;
    writer.flush() catch return UpdateError.WriteFilesFailed;
}

pub fn copySymlinkTo(machine: *UpdateMachine, source_path: [:0]const u8, dest_path: [:0]const u8) UpdateError!void {
    var link_buf: [std.fs.max_path_bytes]u8 = undefined;

    const len = std.Io.Dir.readLinkAbsolute(machine.io, source_path, &link_buf) catch return UpdateError.WriteFilesFailed;

    const target_c = machine.allocator.dupeZ(u8, link_buf[0..len]) catch return UpdateError.AllocZFailed;
    defer machine.allocator.free(target_c);

    std.Io.Dir.deleteFileAbsolute(machine.io, dest_path) catch {};

    const symlink_create_result = std.os.linux.syscall3(.symlinkat, @intFromPtr(target_c.ptr), @bitCast(@as(isize, std.c.AT.FDCWD)), @intFromPtr(dest_path.ptr));
    if (std.os.linux.errno(symlink_create_result) != .SUCCESS) return UpdateError.WriteFilesFailed;
}
