// ── Imports ─────────────────────────────────────────────────────────────────────
const installer = @import("installer.zig");
const std = installer.std;
const c_libs = installer.c_libs;

const data = installer.data;

const InstallerMachine = installer.InstallerMachine;
const InstallerError = installer.InstallerError;

// ── Helpers functions ───────────────────────────────────────────────────
// A recursive assistant. It traverses the directory structure, calculates checksums for all files, and populates the FileMap. It is precisely this data that is subsequently written to the `.files` file within the database
pub fn collectFileChecksums(machine: *InstallerMachine, file_map: *data.FileMap) !void {
    const current_entry = machine.data.packages[machine.current_package_index];
    var dir = try machine.check(std.Io.Dir.openDirAbsolute(machine.io, std.mem.span(current_entry.temp_path), .{ .iterate = true }), InstallerError.CollectFileChecksumsFailed);
    defer dir.close(machine.io);

    var walker = try dir.walk(machine.allocator);
    defer walker.deinit();

    while (try walker.next(machine.io)) |entry| {
        if (entry.kind != .file and entry.kind != .sym_link) continue;

        if (machine.cancellable) |cancellable| if (c_libs.g_cancellable_is_cancelled(cancellable) != 0) return InstallerError.Cancelled;

        var hex_buf: [65]u8 = undefined;

        if (entry.kind == .sym_link) {
            var link_target_buf: [std.fs.max_path_bytes]u8 = undefined;
            const link_target_len = entry.dir.readLink(machine.io, entry.basename, &link_target_buf) catch
                return InstallerError.CollectFileChecksumsFailed;
            const link_target = link_target_buf[0..link_target_len];
            var hash_bytes: [std.crypto.hash.sha2.Sha256.digest_length]u8 = undefined;
            std.crypto.hash.sha2.Sha256.hash(link_target, &hash_bytes, .{});
            const hex = std.fmt.bytesToHex(hash_bytes, .lower);
            @memcpy(hex_buf[0..64], &hex);
            hex_buf[64] = 0;
        } else {
            const abs_path = try machine.check(std.fs.path.joinZ(machine.allocator, &.{ std.mem.span(current_entry.temp_path), entry.path }), InstallerError.CollectFileChecksumsFailed);
            defer machine.allocator.free(abs_path);

            const gfile = c_libs.g_file_new_for_path(abs_path.ptr);
            defer c_libs.g_object_unref(@ptrCast(gfile));

            var raw_checksum_bin: [*c]u8 = null;
            if (c_libs.ostree_checksum_file(gfile, c_libs.OSTREE_OBJECT_TYPE_FILE, &raw_checksum_bin, machine.cancellable, &machine.gerror) == 0) {
                if (machine.gerror) |err| {
                    c_libs.g_error_free(err);
                    machine.gerror = null;
                }
                return InstallerError.CollectFileChecksumsFailed;
            }
            defer c_libs.g_free(@ptrCast(raw_checksum_bin));
            c_libs.ostree_checksum_inplace_from_bytes(raw_checksum_bin.?, &hex_buf);
        }

        try machine.check(file_map.put(try machine.allocator.dupe(u8, entry.path), try machine.allocator.dupe(u8, hex_buf[0..64])), InstallerError.CollectFileChecksumsFailed);
    }
}

pub fn dirSize(machine: *InstallerMachine, root_path: []const u8) !u64 {
    var total_size: u64 = 0;

    var dir = std.Io.Dir.openDirAbsolute(machine.io, root_path, .{ .iterate = true }) catch return 0;
    defer dir.close(machine.io);

    var walker = try dir.walk(machine.allocator);
    defer walker.deinit();

    while (try walker.next(machine.io)) |entry| {
        if (entry.kind != .file) continue;
        const stat = entry.dir.statFile(machine.io, entry.basename, .{}) catch continue;
        total_size += stat.size;
    }

    return total_size;
}

pub fn estimateCheckoutSize(machine: *InstallerMachine) !u64 {
    var root_file: ?*c_libs.GFile = null;
    defer if (root_file) |file| c_libs.g_object_unref(@ptrCast(file));

    const repo = try machine.unwrap(machine.repo, InstallerError.RepoOpenFailed);

    if (machine.commit_checksum == null) {
        return InstallerError.CheckSpaceFailed;
    }
    if (c_libs.ostree_repo_read_commit(repo, machine.commit_checksum, &root_file, null, machine.cancellable, &machine.gerror) == 0) return InstallerError.CheckSpaceFailed;

    const root_file_unwraped = try machine.unwrap(root_file, InstallerError.CheckSpaceFailed);

    return walkTree(machine, @ptrCast(root_file_unwraped));
}

fn walkTree(machine: *InstallerMachine, root: *anyopaque) !u64 {
    var total: u64 = 0;

    var queue = std.ArrayList(*anyopaque).empty;
    defer {
        for (queue.items) |item| c_libs.g_object_unref(item);
        queue.deinit(machine.allocator);
    }

    _ = c_libs.g_object_ref(root);
    try queue.append(machine.allocator, root);

    while (queue.items.len > 0) {
        const dir = queue.pop();
        defer c_libs.g_object_unref(dir);

        const enumerator = c_libs.g_file_enumerate_children(@ptrCast(dir), "standard::name,standard::type,standard::size", c_libs.G_FILE_QUERY_INFO_NOFOLLOW_SYMLINKS, machine.cancellable, &machine.gerror) orelse return error.DiffFailed;
        defer c_libs.g_object_unref(enumerator);

        while (true) {
            const info: ?*anyopaque = c_libs.g_file_enumerator_next_file(enumerator, machine.cancellable, &machine.gerror);
            if (info == null) break;
            defer c_libs.g_object_unref(info);

            const file_type = c_libs.g_file_info_get_file_type(@ptrCast(info));
            const child_name = c_libs.g_file_info_get_name(@ptrCast(info));
            const child = c_libs.g_file_get_child(@ptrCast(dir), child_name) orelse continue;

            if (file_type == c_libs.G_FILE_TYPE_DIRECTORY) {
                try queue.append(machine.allocator, child);
            } else {
                defer c_libs.g_object_unref(child);
                total += @intCast(c_libs.g_file_info_get_size(@ptrCast(info)));
            }
        }
    }

    return total;
}

pub fn loadCommitBody(machine: *InstallerMachine, checksum: [*c]const u8) InstallerError![]const u8 {
    const repo = try machine.unwrap(machine.repo, InstallerError.RepoOpenFailed);

    if (checksum == null) return try machine.check(machine.allocator.dupe(u8, ""), InstallerError.AllocZFailed);

    var commit_variant: ?*c_libs.GVariant = null;
    defer if (commit_variant) |variant| c_libs.g_variant_unref(variant);

    if (c_libs.ostree_repo_load_variant(repo, c_libs.OSTREE_OBJECT_TYPE_COMMIT, checksum, &commit_variant, &machine.gerror) == 0) return try machine.check(machine.allocator.dupe(u8, ""), InstallerError.AllocZFailed);

    const commit_body_variant = c_libs.g_variant_get_child_value(commit_variant, 4);
    defer if (commit_body_variant) |variant| c_libs.g_variant_unref(variant);

    var body_len: usize = 0;
    const body_ptr = c_libs.g_variant_get_string(commit_body_variant, &body_len);

    return try machine.check(machine.allocator.dupe(u8, body_ptr[0..body_len]), InstallerError.AllocZFailed);
}

pub fn mergeDirs(source_path: [:0]const u8, dest_path: [:0]const u8, allocator: std.mem.Allocator) !void {
    const io = std.Io.Threaded.global_single_threaded.io();

    const Entry = struct { path: []const u8, is_dir: bool };
    var entries = std.ArrayList(Entry).empty;
    defer {
        for (entries.items) |e| allocator.free(e.path);
        entries.deinit(allocator);
    }

    {
        var source_dir = try std.Io.Dir.openDirAbsolute(io, source_path, .{ .iterate = true });
        defer source_dir.close(io);

        var source_walker = try source_dir.walk(allocator);
        defer source_walker.deinit();

        while (try source_walker.next(io)) |entry| {
            if (entry.kind != .file and entry.kind != .directory) continue;
            try entries.append(allocator, .{
                .path = try allocator.dupe(u8, entry.path),
                .is_dir = entry.kind == .directory,
            });
        }
    }

    for (entries.items) |entry| {
        if (!entry.is_dir) continue;
        const dest_child = try std.fs.path.joinZ(allocator, &.{ dest_path, entry.path });
        defer allocator.free(dest_child);
        std.Io.Dir.cwd().createDirPath(io, dest_child) catch {};
    }

    for (entries.items) |entry| {
        if (entry.is_dir) continue;
        const source_child = try std.fs.path.joinZ(allocator, &.{ source_path, entry.path });
        defer allocator.free(source_child);
        const dest_child = try std.fs.path.joinZ(allocator, &.{ dest_path, entry.path });
        defer allocator.free(dest_child);

        const result = std.os.linux.syscall4(
            .renameat,
            @bitCast(@as(isize, std.c.AT.FDCWD)),
            @intFromPtr(source_child.ptr),
            @bitCast(@as(isize, std.c.AT.FDCWD)),
            @intFromPtr(dest_child.ptr),
        );
        if (std.os.linux.errno(result) != .SUCCESS) return error.MergeFailed;
    }

    try std.Io.Dir.cwd().deleteTree(io, source_path);
}

pub fn mirrorDir(machine: *InstallerMachine, source_path: [:0]const u8, dest_path: [:0]const u8) !void {
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

pub fn overlayDir(machine: *InstallerMachine, source_path: [:0]const u8, dest_path: [:0]const u8) !void {
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

pub fn copyFileTo(machine: *InstallerMachine, source_path: [:0]const u8, dest_path: []const u8) !void {
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
