const std = @import("std");

const c_libs = @import("c-libs");

const uninstaller = @import("../uninstaller.zig");
const UninstallerMachine = uninstaller.UninstallerMachine;
const UninstallerError = uninstaller.UninstallerError;

fn copyFileTo(machine: *UninstallerMachine, src: [:0]const u8, dst: [:0]const u8) UninstallerError!void {
    const content = std.Io.Dir.cwd().readFileAlloc(machine.io, src, machine.allocator, .limited(10 * 1024 * 1024)) catch return UninstallerError.AllocZFailed;
    defer machine.allocator.free(content);

    const file = std.Io.Dir.createFileAbsolute(machine.io, dst, .{}) catch return UninstallerError.AllocZFailed;
    defer file.close(machine.io);

    var buf: [4096]u8 = undefined;
    var bw = file.writer(machine.io, &buf);
    bw.interface.writeAll(content) catch return UninstallerError.AllocZFailed;
    bw.interface.flush() catch return UninstallerError.AllocZFailed;
}

fn copySymlinkTo(machine: *UninstallerMachine, src: [:0]const u8, dst: [:0]const u8) UninstallerError!void {
    var link_buf: [std.fs.max_path_bytes]u8 = undefined;
    const len = std.Io.Dir.readLinkAbsolute(machine.io, src, &link_buf) catch return UninstallerError.AllocZFailed;

    const target = machine.allocator.dupeZ(u8, link_buf[0..len]) catch return UninstallerError.AllocZFailed;
    defer machine.allocator.free(target);

    std.Io.Dir.deleteFileAbsolute(machine.io, dst) catch {};
    const result = std.os.linux.syscall3(.symlinkat, @intFromPtr(target.ptr), @bitCast(@as(isize, std.c.AT.FDCWD)), @intFromPtr(dst.ptr));
    if (std.os.linux.errno(result) != .SUCCESS) return UninstallerError.AllocZFailed;
}

pub fn mirrorDir(machine: *UninstallerMachine, src: [:0]const u8, dst: [:0]const u8) UninstallerError!void {
    var dir = std.Io.Dir.openDirAbsolute(machine.io, src, .{ .iterate = true }) catch return;
    defer dir.close(machine.io);

    var walker = dir.walk(machine.allocator) catch return UninstallerError.AllocZFailed;
    defer walker.deinit();

    while (walker.next(machine.io) catch return UninstallerError.AllocZFailed) |entry| {
        const dst_child = std.fs.path.joinZ(machine.allocator, &.{ dst, entry.path }) catch return UninstallerError.AllocZFailed;
        defer machine.allocator.free(dst_child);

        switch (entry.kind) {
            .directory => std.Io.Dir.cwd().createDirPath(machine.io, dst_child) catch {},
            .file => {
                const src_child = std.fs.path.joinZ(machine.allocator, &.{ src, entry.path }) catch continue;
                defer machine.allocator.free(src_child);
                copyFileTo(machine, src_child, dst_child) catch {};
            },
            .sym_link => {
                const src_child = std.fs.path.joinZ(machine.allocator, &.{ src, entry.path }) catch continue;
                defer machine.allocator.free(src_child);
                copySymlinkTo(machine, src_child, dst_child) catch {};
            },
            else => {},
        }
    }
}

pub fn computeLiveChecksum(machine: *UninstallerMachine, abs_path: [:0]const u8) UninstallerError![32]u8 {
    var hash_bytes: [std.crypto.hash.sha2.Sha256.digest_length]u8 = undefined;

    var link_buf: [std.fs.max_path_bytes]u8 = undefined;
    if (std.Io.Dir.readLinkAbsolute(machine.io, abs_path, &link_buf)) |len| {
        std.crypto.hash.sha2.Sha256.hash(link_buf[0..len], &hash_bytes, .{});
        return hash_bytes;
    } else |_| {}

    const file = std.Io.Dir.openFileAbsolute(machine.io, abs_path, .{}) catch return UninstallerError.FileNotFound;
    defer file.close(machine.io);

    var file_buf: [4096]u8 = undefined;
    var hash_buf: [4096]u8 = undefined;
    var reader = file.reader(machine.io, &file_buf);
    var hasher = reader.interface.hashed(std.crypto.hash.sha2.Sha256.init(.{}), &hash_buf);
    _ = hasher.reader.discardRemaining() catch return UninstallerError.FileMapCorrupted;
    hasher.hasher.final(&hash_bytes);

    return hash_bytes;
}

pub fn removeEmptyDirs(machine: *UninstallerMachine, root: [:0]const u8) void {
    var dir = std.Io.Dir.openDirAbsolute(machine.io, root, .{ .iterate = true }) catch return;

    var subdirs = std.ArrayList([:0]u8).empty;
    var iter = dir.iterate();
    while (iter.next(machine.io) catch {
        dir.close(machine.io);
        return;
    }) |entry| {
        if (entry.kind != .directory) continue;
        const child = std.fs.path.joinZ(machine.allocator, &.{ root, entry.name }) catch continue;
        subdirs.append(machine.allocator, child) catch machine.allocator.free(child);
    }
    dir.close(machine.io);

    defer {
        for (subdirs.items) |sub_dir| machine.allocator.free(sub_dir);
        subdirs.deinit(machine.allocator);
    }

    for (subdirs.items) |child| {
        removeEmptyDirs(machine, child);
        _ = std.os.linux.syscall1(.rmdir, @intFromPtr(child.ptr));
    }
}
