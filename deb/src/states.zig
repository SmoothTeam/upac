// ── Imports ─────────────────────────────────────────────────────────────────────
const backend = @import("backend.zig");
const std = backend.std;
const c_libs = backend.c_libs;

const Machine = backend.BackendMachine;
const PackageMeta = backend.PackageMeta;
const package_meta_field_map = backend.package_meta_field_map;

const BackendError = backend.BackendError;
const StateId = backend.StateId;

const utils = @import("utils.zig");
const parseLicenseFromCopyright = utils.parseLicenseFromCopyright;

const copyArchiveEntry = utils.copyArchiveEntry;

const computeMd5 = utils.computeMd5;

const isControlFile = utils.isControlFile;
const isCopyrightFile = utils.isCopyrightFile;

const readFileFromNestedTar = utils.readFileFromNestedTar;

// ── Trampoline ────────────────────────────────────────────────────────────────
pub fn stateStart(machine: *Machine) BackendError!void {
    var state = StateId.verifying;
    while (state != .done) {
        try machine.enter(state);
        state = switch (state) {
            .verifying => try stateVerifying(machine),
            .extracting => try stateExtracting(machine),
            .special_step => try stateVerifyingFiles(machine),
            .reading_meta => try stateReadingMeta(machine),
            .done, .failed => unreachable,
        };
    }
    try machine.enter(.done);
}

// ── States ─────────────────────────────────────────────────────────────────
// Archive integrity check status: calculating SHA256 and comparing against expected value
fn stateVerifying(machine: *Machine) BackendError!StateId {
    var hasher = std.crypto.hash.sha2.Sha256.init(.{});
    var hasher_buf: [65536]u8 = undefined;

    var digest: [std.crypto.hash.sha2.Sha256.digest_length]u8 = undefined;
    var expected_bytes: [std.crypto.hash.sha2.Sha256.digest_length]u8 = undefined;

    const package_file = try machine.check(std.Io.Dir.openFileAbsolute(machine.io, std.mem.span(machine.request.pkg_path), .{}), BackendError.ReadFailed);
    machine.file = package_file;

    while (true) {
        if (machine.isCancelRequested()) {
            stateFailed(machine);
            return BackendError.Cancelled;
        }
        const iov = [1][]u8{hasher_buf[0..]};
        const index = try machine.check(package_file.readStreaming(machine.io, &iov), BackendError.ReadFailed);

        if (index == 0) break;
        hasher.update(hasher_buf[0..index]);
    }
    hasher.final(&digest);

    _ = try machine.check(std.fmt.hexToBytes(&expected_bytes, machine.request.checksum.ptr[0..machine.request.checksum.len]), BackendError.InvalidPackage);

    if (!std.mem.eql(u8, &digest, &expected_bytes)) {
        stateFailed(machine);
        return BackendError.ChecksumMismatch;
    }

    const file_descriptor = try machine.unwrap(machine.file, BackendError.ArchiveOpenFailed);
    try machine.check(machine.io.vtable.fileSeekTo(machine.io.userdata, file_descriptor, 0), BackendError.ArchiveOpenFailed);

    return .extracting;
}

// Unpacking state: uses libarchive to extract files to the temp directory
fn stateExtracting(machine: *Machine) BackendError!StateId {
    var tem_dir_buf: [256]u8 = undefined;
    const timestamp = @divTrunc(std.Io.Clock.real.now(machine.io).nanoseconds, std.time.ns_per_ms);

    const tepm_dir_name = try machine.check(std.fmt.bufPrintZ(&tem_dir_buf, "upac-installed-{d}", .{timestamp}), BackendError.AllocZFailed);
    const temp_dir_path = try machine.check(std.Io.Dir.path.joinZ(machine.allocator, &.{ std.mem.span(machine.request.temp_dir), tepm_dir_name }), BackendError.AllocZFailed);

    try machine.check(std.Io.Dir.createDirAbsolute(machine.io, temp_dir_path, .default_file), BackendError.TempDirFailed);
    machine.temp_path = temp_dir_path;

    const archive_reader = try machine.unwrap(c_libs.archive_read_new(), BackendError.ArchiveOpenFailed);
    defer _ = c_libs.archive_read_free(archive_reader);

    _ = c_libs.archive_read_support_format_ar(archive_reader);
    _ = c_libs.archive_read_support_filter_all(archive_reader);

    if (c_libs.archive_read_open_filename(archive_reader, machine.request.pkg_path, 16384) != c_libs.ARCHIVE_OK) {
        stateFailed(machine);
        return BackendError.ArchiveOpenFailed;
    }

    const archive_writer = c_libs.archive_write_disk_new() orelse {
        stateFailed(machine);
        return BackendError.ArchiveOpenFailed;
    };
    defer _ = c_libs.archive_write_free(archive_writer);

    _ = c_libs.archive_write_disk_set_options(
        archive_writer,
        c_libs.ARCHIVE_EXTRACT_TIME |
            c_libs.ARCHIVE_EXTRACT_PERM |
            c_libs.ARCHIVE_EXTRACT_FFLAGS,
    );
    _ = c_libs.archive_write_disk_set_standard_lookup(archive_writer);

    var cwd_buf: [std.Io.Dir.max_path_bytes]u8 = undefined;
    const cwd_len = std.Io.Dir.cwd().realPath(machine.io, &cwd_buf) catch {
        stateFailed(machine);
        return BackendError.OutOfMemory;
    };

    var old_dir = try machine.check(std.Io.Dir.openDirAbsolute(machine.io, cwd_buf[0..cwd_len], .{}), BackendError.ReadFailed);
    defer old_dir.close(machine.io);

    try machine.check(std.Io.Threaded.chdir(temp_dir_path), BackendError.OutOfMemory);
    defer std.Io.Threaded.fchdir(old_dir.handle) catch {};

    var entry: ?*c_libs.archive_entry = null;
    while (c_libs.archive_read_next_header(archive_reader, &entry) == c_libs.ARCHIVE_OK) {
        if (machine.isCancelRequested()) {
            stateFailed(machine);
            return BackendError.Cancelled;
        }

        const entry_name = std.mem.span(c_libs.archive_entry_pathname(entry));

        if (std.mem.startsWith(u8, entry_name, "data.tar")) {
            const size = @as(usize, @intCast(c_libs.archive_entry_size(entry)));
            const data_tar_buffer = try machine.check(machine.allocator.alloc(u8, size), BackendError.OutOfMemory);
            defer machine.allocator.free(data_tar_buffer);

            if (c_libs.archive_read_data(archive_reader, data_tar_buffer.ptr, size) < 0) {
                stateFailed(machine);
                return BackendError.ArchiveReadFailed;
            }

            const inner_archive_reader = try machine.unwrap(c_libs.archive_read_new(), BackendError.ArchiveOpenFailed);
            defer _ = c_libs.archive_read_free(inner_archive_reader);

            _ = c_libs.archive_read_support_format_tar(inner_archive_reader);
            _ = c_libs.archive_read_support_filter_all(inner_archive_reader);

            if (c_libs.archive_read_open_memory(inner_archive_reader, data_tar_buffer.ptr, size) != c_libs.ARCHIVE_OK) {
                stateFailed(machine);
                return BackendError.ArchiveOpenFailed;
            }

            var inner_entry: ?*c_libs.archive_entry = null;
            while (c_libs.archive_read_next_header(inner_archive_reader, &inner_entry) == c_libs.ARCHIVE_OK) {
                if (machine.isCancelRequested()) {
                    stateFailed(machine);
                    return BackendError.Cancelled;
                }
                if (c_libs.archive_write_header(archive_writer, inner_entry) != c_libs.ARCHIVE_OK) {
                    stateFailed(machine);
                    return BackendError.ArchiveExtractFailed;
                }
                try copyArchiveEntry(inner_archive_reader, archive_writer, machine);
            }
        }
    }

    return .special_step;
}

// Verifies the checksums of files listed in md5sums against their actual contents on disk
fn stateVerifyingFiles(machine: *Machine) BackendError!StateId {
    machine.reportDetail("verifying archive integrity...");

    var temp_dir = std.Io.Dir.openDirAbsolute(machine.io, std.mem.span(machine.request.temp_dir), .{}) catch {
        stateFailed(machine);
        return BackendError.ReadFailed;
    };
    defer temp_dir.close(machine.io);

    const md5sums_file = temp_dir.openFile(machine.io, "md5sums", .{}) catch |err| {
        if (err == error.FileNotFound) return .reading_meta;
        stateFailed(machine);
        return BackendError.ReadFailed;
    };
    defer md5sums_file.close(machine.io);

    const content = blk: {
        var list = std.ArrayList(u8).empty;
        errdefer list.deinit(machine.allocator);
        var read_buf: [4096]u8 = undefined;
        while (true) {
            const iov = [1][]u8{read_buf[0..]};
            const n = md5sums_file.readStreaming(machine.io, &iov) catch {
                stateFailed(machine);
                return BackendError.ReadFailed;
            };
            if (n == 0) break;
            list.appendSlice(machine.allocator, read_buf[0..n]) catch {
                stateFailed(machine);
                return BackendError.OutOfMemory;
            };
        }
        break :blk list.toOwnedSlice(machine.allocator) catch {
            stateFailed(machine);
            return BackendError.OutOfMemory;
        };
    };
    defer machine.allocator.free(content);

    var io_buf: [4096]u8 = undefined;
    var lines = std.mem.splitScalar(u8, content, '\n');

    while (lines.next()) |line| {
        if (machine.isCancelRequested()) {
            stateFailed(machine);
            return BackendError.Cancelled;
        }
        const trimmed = std.mem.trim(u8, line, " \r");
        if (trimmed.len == 0) continue;

        var tokens = std.mem.tokenizeAny(u8, trimmed, " \t");
        const expected_hex = tokens.next() orelse continue;
        const file_path = std.mem.trim(u8, tokens.rest(), " \t");

        const file = try machine.check(temp_dir.openFile(machine.io, file_path, .{}), BackendError.ReadFailed);
        defer file.close(machine.io);

        const digest = try machine.check(computeMd5(machine.io, file, &io_buf), BackendError.ReadFailed);
        const actual_hex = std.fmt.bytesToHex(digest, .lower);

        if (!std.mem.eql(u8, &actual_hex, expected_hex)) {
            stateFailed(machine);
            return BackendError.ChecksumMismatch;
        }
    }

    return .reading_meta;
}

// Extracts package metadata from the nested control.tar archive and parses the control file
fn stateReadingMeta(machine: *Machine) BackendError!StateId {
    const archive_reader = try machine.unwrap(c_libs.archive_read_new(), BackendError.ArchiveOpenFailed);
    defer _ = c_libs.archive_read_free(archive_reader);
    _ = c_libs.archive_read_support_format_ar(archive_reader);
    _ = c_libs.archive_read_support_filter_all(archive_reader);

    if (c_libs.archive_read_open_filename(archive_reader, machine.request.pkg_path, 16384) != c_libs.ARCHIVE_OK) {
        stateFailed(machine);
        return BackendError.ArchiveOpenFailed;
    }

    var control_content: ?[]u8 = null;
    defer if (control_content) |c| machine.allocator.free(c);

    var copyright_content: ?[]u8 = null;
    defer if (copyright_content) |c| machine.allocator.free(c);

    var entry: ?*c_libs.archive_entry = null;
    outer: while (c_libs.archive_read_next_header(archive_reader, &entry) == c_libs.ARCHIVE_OK) {
        const entry_name = std.mem.span(c_libs.archive_entry_pathname(entry));

        if (std.mem.startsWith(u8, entry_name, "control.tar")) {
            control_content = try readFileFromNestedTar(machine, archive_reader, entry, isControlFile);
        } else if (std.mem.startsWith(u8, entry_name, "data.tar")) {
            copyright_content = try readFileFromNestedTar(machine, archive_reader, entry, isCopyrightFile);
        }

        if (control_content != null and copyright_content != null) break :outer;
    }

    if (control_content == null) {
        stateFailed(machine);
        return BackendError.InvalidPackage;
    }

    var name: ?[]const u8 = null;
    var version: ?[]const u8 = null;
    var size: u32 = 0;
    var architecture: ?[]const u8 = null;
    var description: ?[]const u8 = null;
    var url: ?[]const u8 = null;
    var packager: ?[]const u8 = null;

    var lines = std.mem.splitScalar(u8, control_content.?, '\n');
    while (lines.next()) |line| {
        const trimmed = std.mem.trim(u8, line, " \t\r");
        if (trimmed.len == 0) continue;

        const separator_index = std.mem.indexOf(u8, trimmed, ": ") orelse continue;
        const key = trimmed[0..separator_index];
        const value = std.mem.trim(u8, trimmed[separator_index + 2 ..], " \t");

        const field = package_meta_field_map.get(key) orelse continue;
        switch (field) {
            .Package => name = try machine.allocator.dupe(u8, value),
            .Version => version = try machine.allocator.dupe(u8, value),
            .@"Installed-Size" => size = std.fmt.parseInt(u32, value, 10) catch 0,
            .Architecture => architecture = try machine.allocator.dupe(u8, value),
            .Description => description = try machine.allocator.dupe(u8, value),
            .Homepage => url = try machine.allocator.dupe(u8, value),
            .Maintainer => packager = try machine.allocator.dupe(u8, value),
        }
    }

    machine.meta = PackageMeta{
        .name = try machine.unwrap(name, BackendError.MetadataNotFound),
        .version = try machine.unwrap(version, BackendError.MetadataNotFound),
        .author = packager orelse try machine.allocator.dupe(u8, "Unknown"),
        .size = size,
        .architecture = architecture orelse try machine.allocator.dupe(u8, "No architecture"),
        .description = description orelse try machine.allocator.dupe(u8, "No description"),
        .license = try parseLicenseFromCopyright(copyright_content, machine.allocator),
        .url = url orelse try machine.allocator.dupe(u8, "No url"),
        .packager = packager orelse try machine.allocator.dupe(u8, "Unknown"),
        .installed_at = @intCast(@divTrunc(std.Io.Clock.real.now(machine.io).nanoseconds, std.time.ns_per_s)),
        .checksum = try machine.allocator.dupe(u8, machine.request.checksum),
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
