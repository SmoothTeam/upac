const std = @import("std");

const c_libs = @import("c-libs");

const uninstaller = @import("../uninstaller.zig");
const UninstallerMachine = uninstaller.UninstallerMachine;
const UninstallerError = uninstaller.UninstallerError;

const MergeMachine = @import("merge.zig").MergeMachine;

pub fn loadParentBody(machine: *MergeMachine) UninstallerError![]const u8 {
    const repo = machine.repo orelse return UninstallerError.RepoOpenFailed;

    var head_checksum: [*c]u8 = null;
    defer c_libs.g_free(head_checksum);

    if (c_libs.ostree_repo_resolve_rev(repo, machine.uninstaller.data.branch, 0, &head_checksum, &machine.uninstaller.gerror) == 0) return UninstallerError.CommitNotFound;
    if (head_checksum == null) return UninstallerError.CommitNotFound;

    var head_variant: ?*c_libs.GVariant = null;
    defer if (head_variant) |variant| c_libs.g_variant_unref(variant);

    if (c_libs.ostree_repo_load_variant(repo, c_libs.OSTREE_OBJECT_TYPE_COMMIT, head_checksum, &head_variant, &machine.uninstaller.gerror) == 0) return UninstallerError.CommitNotFound;

    const parent_bytes_variant = c_libs.g_variant_get_child_value(head_variant, 1) orelse return UninstallerError.CommitNotFound;
    defer c_libs.g_variant_unref(parent_bytes_variant);

    var n_bytes: usize = 0;
    const parent_raw = c_libs.g_variant_get_fixed_array(parent_bytes_variant, &n_bytes, 1);
    if (n_bytes != 32) return UninstallerError.CommitNotFound;

    const parent_bytes: *const [32]u8 = @ptrCast(parent_raw);
    var parent_checksum: [65:0]u8 = std.mem.zeroes([65:0]u8);
    @memcpy(parent_checksum[0..64], &std.fmt.bytesToHex(parent_bytes.*, .lower));

    var parent_variant: ?*c_libs.GVariant = null;
    defer if (parent_variant) |v| c_libs.g_variant_unref(v);

    if (c_libs.ostree_repo_load_variant(repo, c_libs.OSTREE_OBJECT_TYPE_COMMIT, &parent_checksum, &parent_variant, &machine.uninstaller.gerror) == 0) return UninstallerError.CommitNotFound;

    const body_variant = c_libs.g_variant_get_child_value(parent_variant, 4) orelse return UninstallerError.CommitNotFound;
    defer c_libs.g_variant_unref(body_variant);

    var body_len: usize = 0;
    const body_ptr = c_libs.g_variant_get_string(body_variant, &body_len);
    if (body_len == 0) return UninstallerError.CommitNotFound;

    return machine.uninstaller.allocator.dupe(u8, body_ptr[0..body_len]) catch UninstallerError.AllocZFailed;
}

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

pub fn computeLiveChecksum(machine: *UninstallerMachine, abs_path: [:0]const u8) UninstallerError![]const u8 {
    var link_buf: [std.fs.max_path_bytes]u8 = undefined;
    if (std.Io.Dir.readLinkAbsolute(machine.io, abs_path, &link_buf)) |len| {
        var hash_bytes: [std.crypto.hash.sha2.Sha256.digest_length]u8 = undefined;
        std.crypto.hash.sha2.Sha256.hash(link_buf[0..len], &hash_bytes, .{});
        const hex = std.fmt.bytesToHex(hash_bytes, .lower);
        return machine.allocator.dupe(u8, &hex) catch UninstallerError.AllocZFailed;
    } else |_| {}

    const file = std.Io.Dir.openFileAbsolute(machine.io, abs_path, .{}) catch return UninstallerError.FileNotFound;
    defer file.close(machine.io);

    var hash_bytes: [std.crypto.hash.sha2.Sha256.digest_length]u8 = undefined;
    var file_buf: [4096]u8 = undefined;
    var hash_buf: [4096]u8 = undefined;
    var reader = file.reader(machine.io, &file_buf);
    var hasher = reader.interface.hashed(std.crypto.hash.sha2.Sha256.init(.{}), &hash_buf);
    _ = hasher.reader.discardRemaining() catch return UninstallerError.FileMapCorrupted;
    hasher.hasher.final(&hash_bytes);

    const hex = std.fmt.bytesToHex(hash_bytes, .lower);
    return machine.allocator.dupe(u8, &hex) catch UninstallerError.AllocZFailed;
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
