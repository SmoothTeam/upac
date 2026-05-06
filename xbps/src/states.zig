// ── Imports ─────────────────────────────────────────────────────────────────────
const backend = @import("backend.zig");
const std = backend.std;
const c_libs = backend.c_libs;

const Machine = backend.BackendMachine;
const PackageMeta = backend.PackageMeta;
const BackendError = backend.BackendError;
const StateId = backend.StateId;

const plist = @import("parser.zig");

// ── Trampoline ────────────────────────────────────────────────────────────────
pub fn stateStart(machine: *Machine) BackendError!void {
    var state = StateId.verifying;
    while (state != .done) {
        try machine.enter(state);
        state = switch (state) {
            .verifying => try stateVerifying(machine),
            .extracting => try stateExtracting(machine),
            .reading_meta => try stateReadingMeta(machine),
            .done, .failed, .special_step => unreachable,
        };
    }
    try machine.enter(.done);
}

// ── States ─────────────────────────────────────────────────────────────────
fn stateVerifying(machine: *Machine) BackendError!StateId {
    var hasher = std.crypto.hash.sha2.Sha256.init(.{});
    var hasher_buf: [65536]u8 = undefined;

    var digest: [std.crypto.hash.sha2.Sha256.digest_length]u8 = undefined;
    var expected_bytes: [std.crypto.hash.sha2.Sha256.digest_length]u8 = undefined;

    const package_file = try machine.check(std.Io.Dir.openFileAbsolute(machine.io, std.mem.span(machine.request.package_path), .{}), BackendError.ReadFailed);
    machine.file = package_file;

    while (true) {
        if (machine.isCancelRequested()) {
            stateFailed(machine);
            return BackendError.Cancelled;
        }
        const iov = [1][]u8{hasher_buf[0..]};
        const n = try machine.check(package_file.readStreaming(machine.io, &iov), BackendError.ReadFailed);
        if (n == 0) break;
        hasher.update(hasher_buf[0..n]);
    }
    hasher.final(&digest);

    _ = try machine.check(std.fmt.hexToBytes(&expected_bytes, machine.request.checksum), BackendError.InvalidPackage);

    if (!std.mem.eql(u8, &digest, &expected_bytes)) {
        stateFailed(machine);
        return BackendError.ChecksumMismatch;
    }

    const fd = try machine.unwrap(machine.file, BackendError.ArchiveOpenFailed);
    try machine.check(machine.io.vtable.fileSeekTo(machine.io.userdata, fd, 0), BackendError.ArchiveOpenFailed);

    return .extracting;
}

fn stateExtracting(machine: *Machine) BackendError!StateId {
    const fd = try machine.unwrap(machine.file, BackendError.ArchiveOpenFailed);

    var dir_name_buf: [256]u8 = undefined;
    const timestamp = @divTrunc(std.Io.Clock.real.now(machine.io).nanoseconds, std.time.ns_per_ms);
    const dir_name = try machine.check(std.fmt.bufPrintZ(&dir_name_buf, "upac-installed-{d}", .{timestamp}), BackendError.AllocZFailed);
    const temp_dir_path = try machine.check(std.Io.Dir.path.joinZ(machine.allocator, &.{ std.mem.span(machine.request.temp_dir), dir_name }), BackendError.AllocZFailed);

    try machine.check(std.Io.Dir.createDirAbsolute(machine.io, temp_dir_path, .default_file), BackendError.TempDirFailed);
    machine.temp_path = temp_dir_path;

    const reader = try machine.unwrap(c_libs.archive_read_new(), BackendError.ArchiveOpenFailed);
    defer _ = c_libs.archive_read_free(reader);

    _ = c_libs.archive_read_support_format_tar(reader);
    _ = c_libs.archive_read_support_filter_zstd(reader);
    _ = c_libs.archive_read_support_filter_xz(reader);
    _ = c_libs.archive_read_support_filter_gzip(reader);

    _ = c_libs.archive_read_open_fd(reader, fd.handle, 16384);

    const writer = c_libs.archive_write_disk_new() orelse {
        stateFailed(machine);
        return BackendError.ArchiveOpenFailed;
    };
    defer _ = c_libs.archive_write_free(writer);

    _ = c_libs.archive_write_disk_set_options(writer, c_libs.ARCHIVE_EXTRACT_TIME |
        c_libs.ARCHIVE_EXTRACT_PERM |
        c_libs.ARCHIVE_EXTRACT_FFLAGS);
    _ = c_libs.archive_write_disk_set_standard_lookup(writer);

    var cwd_buf: [std.Io.Dir.max_path_bytes]u8 = undefined;
    const cwd_len = std.Io.Dir.cwd().realPath(machine.io, &cwd_buf) catch {
        stateFailed(machine);
        return BackendError.TempDirFailed;
    };

    var old_dir = try machine.check(std.Io.Dir.openDirAbsolute(machine.io, cwd_buf[0..cwd_len], .{}), BackendError.ReadFailed);
    defer old_dir.close(machine.io);

    try machine.check(std.Io.Threaded.chdir(temp_dir_path), BackendError.OutOfMemory);
    defer std.Io.Threaded.fchdir(old_dir.handle) catch {};

    while (true) {
        if (machine.isCancelRequested()) {
            stateFailed(machine);
            return BackendError.Cancelled;
        }

        var entry: ?*c_libs.archive_entry = null;
        const rc = c_libs.archive_read_next_header(reader, &entry);
        if (rc == c_libs.ARCHIVE_EOF) break;
        if (rc != c_libs.ARCHIVE_OK) {
            stateFailed(machine);
            return BackendError.ArchiveReadFailed;
        }

        const entry_path = c_libs.archive_entry_pathname(entry.?);
        const entry_name = if (entry_path != null) std.mem.span(entry_path) else "";

        // Strip leading "./" for matching
        const name = if (std.mem.startsWith(u8, entry_name, "./")) entry_name[2..] else entry_name;

        if (std.mem.eql(u8, name, "props.plist")) {
            const entry_size: usize = @intCast(c_libs.archive_entry_size(entry.?));
            const buf = try machine.check(machine.allocator.alloc(u8, entry_size), BackendError.OutOfMemory);

            if (c_libs.archive_read_data(reader, buf.ptr, entry_size) < 0) {
                machine.allocator.free(buf);
                stateFailed(machine);
                return BackendError.ArchiveReadFailed;
            }
            machine.props_content = buf;
            continue;
        }

        // Skip XBPS metadata files — not needed on disk
        if (std.mem.eql(u8, name, "files.plist") or
            std.mem.eql(u8, name, "INSTALL") or
            std.mem.eql(u8, name, "REMOVE"))
        {
            _ = c_libs.archive_read_data_skip(reader);
            continue;
        }

        if (c_libs.archive_write_header(writer, entry) != c_libs.ARCHIVE_OK) {
            stateFailed(machine);
            return BackendError.ArchiveExtractFailed;
        }

        while (true) {
            if (machine.isCancelRequested()) {
                stateFailed(machine);
                return BackendError.Cancelled;
            }

            var block: ?*const anyopaque = null;
            var size: usize = 0;
            var offset: i64 = 0;

            const rd = c_libs.archive_read_data_block(reader, &block, &size, &offset);
            if (rd == c_libs.ARCHIVE_EOF) break;
            if (rd != c_libs.ARCHIVE_OK) {
                stateFailed(machine);
                return BackendError.ArchiveReadFailed;
            }

            if (c_libs.archive_write_data_block(writer, block, size, offset) != c_libs.ARCHIVE_OK) {
                stateFailed(machine);
                return BackendError.ArchiveExtractFailed;
            }
        }

        if (c_libs.archive_write_finish_entry(writer) != c_libs.ARCHIVE_OK) {
            stateFailed(machine);
            return BackendError.ArchiveExtractFailed;
        }
    }

    return .reading_meta;
}

fn stateReadingMeta(machine: *Machine) BackendError!StateId {
    const content = try machine.unwrap(machine.props_content, BackendError.MetadataNotFound);
    defer {
        machine.allocator.free(content);
        machine.props_content = null;
    }

    const parsed = try machine.check(plist.parse(machine.allocator, content), BackendError.InvalidPackage);
    defer parsed.deinit(machine.allocator);

    machine.meta = PackageMeta{
        .name = try machine.check(machine.allocator.dupe(u8, parsed.name orelse return BackendError.MetadataNotFound), BackendError.OutOfMemory),
        .version = try machine.check(machine.allocator.dupe(u8, parsed.version orelse return BackendError.MetadataNotFound), BackendError.OutOfMemory),
        .arch = try machine.check(machine.allocator.dupe(u8, parsed.architecture orelse "noarch"), BackendError.OutOfMemory),
        .author = try machine.check(machine.allocator.dupe(u8, parsed.maintainer orelse ""), BackendError.OutOfMemory),
        .packager = try machine.check(machine.allocator.dupe(u8, parsed.maintainer orelse ""), BackendError.OutOfMemory),
        .description = try machine.check(machine.allocator.dupe(u8, parsed.short_desc orelse ""), BackendError.OutOfMemory),
        .license = try machine.check(machine.allocator.dupe(u8, parsed.license orelse ""), BackendError.OutOfMemory),
        .url = try machine.check(machine.allocator.dupe(u8, parsed.homepage orelse ""), BackendError.OutOfMemory),
        .checksum = try machine.check(machine.allocator.dupe(u8, machine.request.checksum), BackendError.OutOfMemory),
        .size = parsed.installed_size,
        .installed_at = @intCast(@divTrunc(std.Io.Clock.real.now(machine.io).nanoseconds, std.time.ns_per_s)),
    };

    return .done;
}

// An error state signaling that the machine failed to reach the required state at a certain stage
pub fn stateFailed(machine: *Machine) void {
    if (machine.stack.items.len > 0 and machine.stack.getLast() == .failed) return;
    if (machine.temp_path) |path| {
        std.Io.Dir.cwd().deleteTree(machine.io, path) catch {};
        machine.allocator.free(path);
        machine.temp_path = null;
    }
    machine.stack.append(machine.allocator, .failed) catch {};
    machine.report(.failed);
}
