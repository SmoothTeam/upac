const std = @import("std");

const c_libs = @import("c-libs");

const CONFIG_DIR = @import("upac-types").CONFIG_DIR;

const database = @import("upac-database");
const FileMap = database.FileMap;
const freeFileMap = database.freeFileMap;

const rollback = @import("../rollback.zig");
const RollbackMachine = rollback.RollbackMachine;
const RollbackError = rollback.RollbackError;

const MergeMachine = @import("merge.zig").MergeMachine;

// ── Commit body ───────────────────────────────────────────────────────────────
pub fn loadCurrentCommitBody(machine: *MergeMachine) RollbackError![]const u8 {
    var head_checksum: [*c]u8 = null;
    defer c_libs.g_free(head_checksum);

    var head_variant: ?*c_libs.GVariant = null;
    defer if (head_variant) |variant| c_libs.g_variant_unref(variant);

    const repo = machine.repo orelse return RollbackError.RepoOpenFailed;

    if (c_libs.ostree_repo_resolve_rev(repo, machine.rollback.data.branch, 0, &head_checksum, &machine.rollback.gerror) == 0) return RollbackError.CommitNotFound;
    if (head_checksum == null) return RollbackError.CommitNotFound;

    if (c_libs.ostree_repo_load_variant(repo, c_libs.OSTREE_OBJECT_TYPE_COMMIT, head_checksum, &head_variant, &machine.rollback.gerror) == 0) return RollbackError.CommitNotFound;

    const body_variant = c_libs.g_variant_get_child_value(head_variant, 4) orelse return RollbackError.CommitNotFound;
    defer c_libs.g_variant_unref(body_variant);

    var body_len: usize = 0;
    const body_ptr = c_libs.g_variant_get_string(body_variant, &body_len);

    return machine.rollback.allocator.dupe(u8, body_ptr[0..body_len]) catch RollbackError.AllocZFailed;
}

// ── File map ──────────────────────────────────────────────────────────────────
pub fn buildCombinedFileMap(machine: *MergeMachine, db_path: []const u8) RollbackError!FileMap {
    var combined = FileMap.init(machine.rollback.allocator);
    errdefer freeFileMap(&combined, machine.rollback.allocator);

    var pos: usize = 0;
    while (pos < machine.commit_body.len) {
        while (pos < machine.commit_body.len and machine.commit_body[pos] == '\n') pos += 1;
        if (pos >= machine.commit_body.len) break;

        while (pos < machine.commit_body.len and machine.commit_body[pos] != ' ' and machine.commit_body[pos] != '\t' and machine.commit_body[pos] != '\n') pos += 1;
        if (pos >= machine.commit_body.len or machine.commit_body[pos] == '\n') {
            pos += 1;
            continue;
        }
        pos += 1;

        const checksum_start = pos;
        while (pos < machine.commit_body.len and machine.commit_body[pos] != '\n') pos += 1;
        const checksum = std.mem.trim(u8, machine.commit_body[checksum_start..pos], " \t\r");
        if (pos < machine.commit_body.len) pos += 1;

        if (checksum.len == 0) continue;

        var pkg_map = database.readFiles(db_path, checksum, machine.rollback.allocator) catch continue;
        defer freeFileMap(&pkg_map, machine.rollback.allocator);

        var iter = pkg_map.iterator();
        while (iter.next()) |entry| {
            const key = machine.rollback.allocator.dupe(u8, entry.key_ptr.*) catch continue;
            const val = machine.rollback.allocator.dupe(u8, entry.value_ptr.*) catch {
                machine.rollback.allocator.free(key);
                continue;
            };
            combined.put(key, val) catch {
                machine.rollback.allocator.free(key);
                machine.rollback.allocator.free(val);
            };
        }
    }

    return combined;
}

// ── File I/O ──────────────────────────────────────────────────────────────────
pub fn copyFileTo(machine: *RollbackMachine, src: [:0]const u8, dst: [:0]const u8) RollbackError!void {
    const content = std.Io.Dir.cwd().readFileAlloc(machine.io, src, machine.allocator, .limited(10 * 1024 * 1024)) catch return RollbackError.StagingFailed;
    defer machine.allocator.free(content);

    const file = std.Io.Dir.createFileAbsolute(machine.io, dst, .{}) catch return RollbackError.StagingFailed;
    defer file.close(machine.io);

    var buf: [4096]u8 = undefined;
    var bw = file.writer(machine.io, &buf);
    bw.interface.writeAll(content) catch return RollbackError.StagingFailed;
    bw.interface.flush() catch return RollbackError.StagingFailed;
}

pub fn copySymlinkTo(machine: *RollbackMachine, src: [:0]const u8, dst: [:0]const u8) RollbackError!void {
    var link_buf: [std.fs.max_path_bytes]u8 = undefined;
    const len = std.Io.Dir.readLinkAbsolute(machine.io, src, &link_buf) catch return RollbackError.StagingFailed;

    const target = machine.allocator.dupeZ(u8, link_buf[0..len]) catch return RollbackError.AllocZFailed;
    defer machine.allocator.free(target);

    std.Io.Dir.deleteFileAbsolute(machine.io, dst) catch {};
    const result = std.os.linux.syscall3(.symlinkat, @intFromPtr(target.ptr), @bitCast(@as(isize, std.c.AT.FDCWD)), @intFromPtr(dst.ptr));
    if (std.os.linux.errno(result) != .SUCCESS) return RollbackError.StagingFailed;
}

pub fn computeLiveChecksum(machine: *RollbackMachine, abs_path: [:0]const u8) RollbackError![]const u8 {
    var link_buf: [std.fs.max_path_bytes]u8 = undefined;
    if (std.Io.Dir.readLinkAbsolute(machine.io, abs_path, &link_buf)) |len| {
        var hash_bytes: [std.crypto.hash.sha2.Sha256.digest_length]u8 = undefined;
        std.crypto.hash.sha2.Sha256.hash(link_buf[0..len], &hash_bytes, .{});
        const hex = std.fmt.bytesToHex(hash_bytes, .lower);
        return machine.allocator.dupe(u8, &hex) catch RollbackError.AllocZFailed;
    } else |_| {}

    const file = std.Io.Dir.openFileAbsolute(machine.io, abs_path, .{}) catch return RollbackError.StagingFailed;
    defer file.close(machine.io);

    var hash_bytes: [std.crypto.hash.sha2.Sha256.digest_length]u8 = undefined;
    var file_buf: [4096]u8 = undefined;
    var hash_buf: [4096]u8 = undefined;
    var reader = file.reader(machine.io, &file_buf);
    var hasher = reader.interface.hashed(std.crypto.hash.sha2.Sha256.init(.{}), &hash_buf);
    _ = hasher.reader.discardRemaining() catch return RollbackError.StagingFailed;
    hasher.hasher.final(&hash_bytes);

    const hex = std.fmt.bytesToHex(hash_bytes, .lower);
    return machine.allocator.dupe(u8, &hex) catch RollbackError.AllocZFailed;
}

pub fn removeEmptyDirs(machine: *RollbackMachine, root: [:0]const u8) void {
    var empty_dirs = std.ArrayList([:0]u8).empty;
    defer {
        for (empty_dirs.items) |dir| machine.allocator.free(dir);
        empty_dirs.deinit(machine.allocator);
    }

    var dir = std.Io.Dir.openDirAbsolute(machine.io, root, .{ .iterate = true }) catch return;

    var iter = dir.iterate();
    while (iter.next(machine.io) catch {
        dir.close(machine.io);
        return;
    }) |entry| {
        if (entry.kind != .directory) continue;

        const empty_dir = std.fs.path.joinZ(machine.allocator, &.{ root, entry.name }) catch continue;
        empty_dirs.append(machine.allocator, empty_dir) catch machine.allocator.free(empty_dir);
    }
    dir.close(machine.io);

    for (empty_dirs.items) |empty_dir| {
        removeEmptyDirs(machine, empty_dir);
        _ = std.os.linux.syscall1(.rmdir, @intFromPtr(empty_dir.ptr));
    }
}

pub fn mirrorDir(machine: *RollbackMachine, src: [:0]const u8, dst: [:0]const u8) RollbackError!void {
    var dir = std.Io.Dir.openDirAbsolute(machine.io, src, .{ .iterate = true }) catch return;
    defer dir.close(machine.io);

    var walker = dir.walk(machine.allocator) catch return RollbackError.AllocZFailed;
    defer walker.deinit();

    while (walker.next(machine.io) catch return RollbackError.StagingFailed) |entry| {
        const destination_child = std.fs.path.joinZ(machine.allocator, &.{ dst, entry.path }) catch continue;
        defer machine.allocator.free(destination_child);

        const source_child = std.fs.path.joinZ(machine.allocator, &.{ src, entry.path }) catch continue;
        defer machine.allocator.free(source_child);

        switch (entry.kind) {
            .directory => std.Io.Dir.cwd().createDirPath(machine.io, destination_child) catch {},
            .file => copyFileTo(machine, source_child, destination_child) catch {},
            .sym_link => copySymlinkTo(machine, source_child, destination_child) catch {},

            else => {},
        }
    }
}
