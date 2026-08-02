// SPDX-FileCopyrightText: 2026 2026 JustPav
// SPDX-FileCopyrightText: 2026 JustPav
//
// SPDX-License-Identifier: LGPL-3.0-or-later

// ── Imports ───────────────────────────────────────────────────────────────────
const std = @import("std");

const c_libs = @import("c-libs");

const types = @import("upac-backend-types");

const BackendError = types.BackendError;

const PackageMeta = types.PackageMeta;
const RawMeta = types.RawMeta;

const control_field_map = types.control_field_map;

const backend = @import("../backend.zig");
const Machine = backend.BackendMachine;

const utils = @import("utils.zig");
const parseLicenseFromCopyright = utils.parseLicenseFromCopyright;
const parseVersion = utils.parseVersion;

// ── ParsingState ──────────────────────────────────────────────────────────────
const ParsingState = enum {
    open_archive,
    find_tars,
    open_control_archive,
    scan_control_files,
    verify_md5sums,
    open_data_archive,
    scan_copyright_files,
    parse_control,
    build_meta,
    done,
};

// ── ParsingMachine ────────────────────────────────────────────────────────────
const ParsingMachine = struct {
    backend: *Machine,

    archive_reader: ?*c_libs.archive = null,

    control_tar_buf: ?[]u8 = null,
    control_content: ?[]u8 = null,
    control_inner_reader: ?*c_libs.archive = null,

    data_tar_buf: ?[]u8 = null,
    data_inner_reader: ?*c_libs.archive = null,

    md5sums_content: ?[]u8 = null,

    copyright_content: ?[]u8 = null,

    raw_meta: RawMeta = .{},

    fn stateFailed(self: *ParsingMachine, err: BackendError) BackendError {
        if (self.archive_reader) |reader| {
            _ = c_libs.archive_read_free(reader);
            self.archive_reader = null;
        }

        if (self.control_inner_reader) |reader| {
            _ = c_libs.archive_read_free(reader);
            self.control_inner_reader = null;
        }

        if (self.control_tar_buf) |buf| {
            self.backend.allocator.free(buf);
            self.control_tar_buf = null;
        }

        if (self.data_inner_reader) |reader| {
            _ = c_libs.archive_read_free(reader);
            self.data_inner_reader = null;
        }

        if (self.data_tar_buf) |buf| {
            self.backend.allocator.free(buf);
            self.data_tar_buf = null;
        }

        if (self.control_content) |content| {
            self.backend.allocator.free(content);
            self.control_content = null;
        }

        if (self.md5sums_content) |content| {
            self.backend.allocator.free(content);
            self.md5sums_content = null;
        }

        if (self.copyright_content) |content| {
            self.backend.allocator.free(content);
            self.copyright_content = null;
        }

        self.raw_meta.deinit(self.backend.allocator);

        return err;
    }
};

// ── Trampoline ────────────────────────────────────────────────────────────────
pub fn run(machine: *Machine) BackendError!void {
    var parsing = ParsingMachine{ .backend = machine };

    var state = ParsingState.open_archive;
    while (state != .done) {
        if (machine.data.cancel_token.isCancelled()) return parsing.stateFailed(BackendError.Cancelled);
        state = switch (state) {
            .open_archive => try stateOpenArchive(&parsing),
            .find_tars => try stateFindTars(&parsing),
            .open_control_archive => try stateOpenControlArchive(&parsing),
            .scan_control_files => try stateScanControlFiles(&parsing),
            .verify_md5sums => try stateVerifyMd5sums(&parsing),
            .open_data_archive => try stateOpenDataArchive(&parsing),
            .scan_copyright_files => try stateScanCopyrightFiles(&parsing),
            .parse_control => try stateParseControl(&parsing),
            .build_meta => try stateBuildMeta(&parsing),
            .done => unreachable,
        };
    }
}

// ── States ────────────────────────────────────────────────────────────────────
fn stateOpenArchive(machine: *ParsingMachine) BackendError!ParsingState {
    const archive_reader = c_libs.archive_read_new() orelse return machine.stateFailed(BackendError.ArchiveOpenFailed);
    machine.archive_reader = archive_reader;

    _ = c_libs.archive_read_support_format_ar(archive_reader);
    _ = c_libs.archive_read_support_filter_all(archive_reader);

    if (c_libs.archive_read_open_filename(archive_reader, machine.backend.data.package_path_c, 16384) != c_libs.ARCHIVE_OK) return machine.stateFailed(BackendError.ArchiveOpenFailed);

    return .find_tars;
}

fn stateFindTars(machine: *ParsingMachine) BackendError!ParsingState {
    const archive_reader = machine.archive_reader orelse return machine.stateFailed(BackendError.ArchiveOpenFailed);

    var outer_entry: ?*c_libs.archive_entry = null;
    const result = c_libs.archive_read_next_header(archive_reader, &outer_entry);

    if (result == c_libs.ARCHIVE_EOF) {
        _ = c_libs.archive_read_free(archive_reader);
        machine.archive_reader = null;

        if (machine.control_tar_buf == null) return machine.stateFailed(BackendError.InvalidPackage);

        return .open_control_archive;
    }
    if (result != c_libs.ARCHIVE_OK) return machine.stateFailed(BackendError.ArchiveReadFailed);

    const entry_name = std.mem.span(c_libs.archive_entry_pathname(outer_entry));
    const raw_size = c_libs.archive_entry_size(outer_entry);

    if (std.mem.startsWith(u8, entry_name, "control.tar") and machine.control_tar_buf == null) {
        if (raw_size <= 0) return machine.stateFailed(BackendError.InvalidPackage);
        const entry_size: usize = @intCast(raw_size);
        const control_buf = machine.backend.allocator.alloc(u8, entry_size) catch return machine.stateFailed(BackendError.OutOfMemory);
        machine.control_tar_buf = control_buf;
        if (c_libs.archive_read_data(archive_reader, control_buf.ptr, entry_size) < 0) return machine.stateFailed(BackendError.ArchiveReadFailed);
    } else if (std.mem.startsWith(u8, entry_name, "data.tar") and machine.data_tar_buf == null) {
        if (raw_size > 0) {
            const entry_size: usize = @intCast(raw_size);
            const data_buf = machine.backend.allocator.alloc(u8, entry_size) catch return machine.stateFailed(BackendError.OutOfMemory);
            machine.data_tar_buf = data_buf;
            if (c_libs.archive_read_data(archive_reader, data_buf.ptr, entry_size) < 0) return machine.stateFailed(BackendError.ArchiveReadFailed);
        }
    }

    if (machine.control_tar_buf != null and machine.data_tar_buf != null) {
        _ = c_libs.archive_read_free(archive_reader);
        machine.archive_reader = null;
        return .open_control_archive;
    }

    return .find_tars;
}

fn stateOpenControlArchive(machine: *ParsingMachine) BackendError!ParsingState {
    const control_tar_buf = machine.control_tar_buf orelse return machine.stateFailed(BackendError.InvalidPackage);

    const inner_reader = c_libs.archive_read_new() orelse return machine.stateFailed(BackendError.ArchiveOpenFailed);
    machine.control_inner_reader = inner_reader;

    _ = c_libs.archive_read_support_format_tar(inner_reader);
    _ = c_libs.archive_read_support_filter_all(inner_reader);

    if (c_libs.archive_read_open_memory(inner_reader, control_tar_buf.ptr, control_tar_buf.len) != c_libs.ARCHIVE_OK) return machine.stateFailed(BackendError.ArchiveOpenFailed);

    return .scan_control_files;
}

fn stateScanControlFiles(machine: *ParsingMachine) BackendError!ParsingState {
    const inner_reader = machine.control_inner_reader orelse return machine.stateFailed(BackendError.ArchiveOpenFailed);

    var inner_entry: ?*c_libs.archive_entry = null;
    const result = c_libs.archive_read_next_header(inner_reader, &inner_entry);

    if (result == c_libs.ARCHIVE_EOF) {
        _ = c_libs.archive_read_free(inner_reader);
        machine.control_inner_reader = null;
        if (machine.control_tar_buf) |buf| {
            machine.backend.allocator.free(buf);
            machine.control_tar_buf = null;
        }
        if (machine.control_content == null) return machine.stateFailed(BackendError.InvalidPackage);
        return .verify_md5sums;
    }
    if (result != c_libs.ARCHIVE_OK) return machine.stateFailed(BackendError.ArchiveReadFailed);

    const entry_name = std.mem.span(c_libs.archive_entry_pathname(inner_entry));
    const name = if (std.mem.startsWith(u8, entry_name, "./")) entry_name[2..] else entry_name;
    const raw_size = c_libs.archive_entry_size(inner_entry);

    if (raw_size > 0) {
        const entry_size: usize = @intCast(raw_size);
        if (std.mem.eql(u8, name, "control") and machine.control_content == null) {
            const content = machine.backend.allocator.alloc(u8, entry_size) catch return machine.stateFailed(BackendError.OutOfMemory);
            machine.control_content = content;
            if (c_libs.archive_read_data(inner_reader, content.ptr, entry_size) < 0) return machine.stateFailed(BackendError.ArchiveReadFailed);
        } else if (std.mem.eql(u8, name, "md5sums") and machine.md5sums_content == null) {
            const content = machine.backend.allocator.alloc(u8, entry_size) catch return machine.stateFailed(BackendError.OutOfMemory);
            machine.md5sums_content = content;
            if (c_libs.archive_read_data(inner_reader, content.ptr, entry_size) < 0) return machine.stateFailed(BackendError.ArchiveReadFailed);
        }
    }

    if (machine.control_content != null and machine.md5sums_content != null) {
        _ = c_libs.archive_read_free(inner_reader);
        machine.control_inner_reader = null;
        if (machine.control_tar_buf) |buf| {
            machine.backend.allocator.free(buf);
            machine.control_tar_buf = null;
        }
        return .verify_md5sums;
    }

    return .scan_control_files;
}

fn stateVerifyMd5sums(machine: *ParsingMachine) BackendError!ParsingState {
    var io_buf: [4096]u8 = undefined;

    const md5sums_content = machine.md5sums_content orelse return .open_data_archive;
    defer machine.backend.allocator.free(md5sums_content);
    machine.md5sums_content = null;

    const temp_package_path = machine.backend.temp_package_path orelse return machine.stateFailed(BackendError.TempDirFailed);

    var temp_dir = std.Io.Dir.openDirAbsolute(machine.backend.io, temp_package_path, .{}) catch return machine.stateFailed(BackendError.ReadFailed);
    defer temp_dir.close(machine.backend.io);

    var lines = std.mem.splitScalar(u8, md5sums_content, '\n');
    while (lines.next()) |line| {
        var hasher = std.crypto.hash.Md5.init(.{});

        const trimmed_line = std.mem.trim(u8, line, " \t\r");
        if (trimmed_line.len == 0) continue;

        var tokens = std.mem.tokenizeAny(u8, trimmed_line, " \t");
        const expected_hex = tokens.next() orelse continue;

        const file_path = std.mem.trim(u8, tokens.rest(), " \t");

        const file = temp_dir.openFile(machine.backend.io, file_path, .{}) catch continue;
        defer file.close(machine.backend.io);

        while (true) {
            const iov = [1][]u8{io_buf[0..]};
            const bytes_read = file.readStreaming(machine.backend.io, &iov) catch |err| {
                if (err == error.EndOfStream) break;

                return machine.stateFailed(BackendError.ReadFailed);
            };

            if (bytes_read == 0) break;
            hasher.update(io_buf[0..bytes_read]);
        }

        var digest_hash: [std.crypto.hash.Md5.digest_length]u8 = undefined;
        hasher.final(&digest_hash);
        const actual_hex = std.fmt.bytesToHex(digest_hash, .lower);

        if (!std.mem.eql(u8, &actual_hex, expected_hex)) return machine.stateFailed(BackendError.ChecksumMismatch);
    }

    return .open_data_archive;
}

fn stateOpenDataArchive(machine: *ParsingMachine) BackendError!ParsingState {
    const data_tar_buf = machine.data_tar_buf orelse return .parse_control;

    const inner_reader = c_libs.archive_read_new() orelse return machine.stateFailed(BackendError.ArchiveOpenFailed);
    machine.data_inner_reader = inner_reader;

    _ = c_libs.archive_read_support_format_tar(inner_reader);
    _ = c_libs.archive_read_support_filter_all(inner_reader);

    if (c_libs.archive_read_open_memory(inner_reader, data_tar_buf.ptr, data_tar_buf.len) != c_libs.ARCHIVE_OK) return machine.stateFailed(BackendError.ArchiveOpenFailed);

    return .scan_copyright_files;
}

fn stateScanCopyrightFiles(machine: *ParsingMachine) BackendError!ParsingState {
    const inner_reader = machine.data_inner_reader orelse return machine.stateFailed(BackendError.ArchiveOpenFailed);

    var inner_entry: ?*c_libs.archive_entry = null;
    const result = c_libs.archive_read_next_header(inner_reader, &inner_entry);

    if (result == c_libs.ARCHIVE_EOF) {
        _ = c_libs.archive_read_free(inner_reader);
        machine.data_inner_reader = null;
        if (machine.data_tar_buf) |buf| {
            machine.backend.allocator.free(buf);
            machine.data_tar_buf = null;
        }
        return .parse_control;
    }
    if (result != c_libs.ARCHIVE_OK) return machine.stateFailed(BackendError.ArchiveReadFailed);

    const entry_name = std.mem.span(c_libs.archive_entry_pathname(inner_entry));
    const name = if (std.mem.startsWith(u8, entry_name, "./")) entry_name[2..] else entry_name;

    if (!std.mem.startsWith(u8, name, "usr/share/doc/") or !std.mem.endsWith(u8, name, "/copyright")) return .scan_copyright_files;

    const raw_size = c_libs.archive_entry_size(inner_entry);
    if (raw_size <= 0) {
        _ = c_libs.archive_read_free(inner_reader);
        machine.data_inner_reader = null;
        if (machine.data_tar_buf) |buf| {
            machine.backend.allocator.free(buf);
            machine.data_tar_buf = null;
        }
        return .parse_control;
    }

    const entry_size: usize = @intCast(raw_size);
    const content = machine.backend.allocator.alloc(u8, entry_size) catch return machine.stateFailed(BackendError.OutOfMemory);
    machine.copyright_content = content;
    if (c_libs.archive_read_data(inner_reader, content.ptr, entry_size) < 0) return machine.stateFailed(BackendError.ArchiveReadFailed);

    _ = c_libs.archive_read_free(inner_reader);
    machine.data_inner_reader = null;
    if (machine.data_tar_buf) |buf| {
        machine.backend.allocator.free(buf);
        machine.data_tar_buf = null;
    }

    return .parse_control;
}

fn stateParseControl(machine: *ParsingMachine) BackendError!ParsingState {
    const control_content = machine.control_content orelse return machine.stateFailed(BackendError.InvalidPackage);
    machine.control_content = null;
    defer machine.backend.allocator.free(control_content);

    var lines = std.mem.splitScalar(u8, control_content, '\n');
    while (lines.next()) |line| {
        const trimmed = std.mem.trim(u8, line, " \t\r");
        if (trimmed.len == 0) continue;

        const separator_index = std.mem.indexOf(u8, trimmed, ": ") orelse continue;
        const key = trimmed[0..separator_index];
        const value = std.mem.trim(u8, trimmed[separator_index + 2 ..], " \t");

        const field_kind = control_field_map.get(key) orelse continue;
        switch (field_kind) {
            .name => machine.raw_meta.name = machine.backend.allocator.dupe(u8, value) catch return machine.stateFailed(BackendError.OutOfMemory),
            .version => machine.raw_meta.version = machine.backend.allocator.dupe(u8, value) catch return machine.stateFailed(BackendError.OutOfMemory),
            .arch => machine.raw_meta.arch = machine.backend.allocator.dupe(u8, value) catch return machine.stateFailed(BackendError.OutOfMemory),
            .size => machine.raw_meta.size = std.fmt.parseInt(u32, value, 10) catch 0,
            .description => machine.raw_meta.description = machine.backend.allocator.dupe(u8, value) catch return machine.stateFailed(BackendError.OutOfMemory),
            .url => machine.raw_meta.url = machine.backend.allocator.dupe(u8, value) catch return machine.stateFailed(BackendError.OutOfMemory),
            .packager => machine.raw_meta.packager = machine.backend.allocator.dupe(u8, value) catch return machine.stateFailed(BackendError.OutOfMemory),
            .license => {},
        }
    }

    if (machine.copyright_content) |copyright| {
        machine.raw_meta.license = parseLicenseFromCopyright(copyright, machine.backend.allocator) catch return machine.stateFailed(BackendError.OutOfMemory);
        machine.backend.allocator.free(copyright);
        machine.copyright_content = null;
    }

    return .build_meta;
}

fn stateBuildMeta(machine: *ParsingMachine) BackendError!ParsingState {
    var sha256: [32]u8 = undefined;

    _ = std.fmt.hexToBytes(&sha256, machine.backend.data.checksum) catch return machine.stateFailed(BackendError.InvalidPackage);

    const raw_version_str = machine.raw_meta.version orelse return machine.stateFailed(BackendError.MetadataNotFound);
    defer machine.backend.allocator.free(raw_version_str);
    machine.raw_meta.version = null;

    const parsed_version = parseVersion(machine.backend.allocator, raw_version_str) catch return machine.stateFailed(BackendError.InvalidPackage);
    errdefer parsed_version.deinit(machine.backend.allocator);

    const package_name = machine.raw_meta.name orelse return machine.stateFailed(BackendError.MetadataNotFound);
    machine.raw_meta.name = null;
    errdefer machine.backend.allocator.free(package_name);

    const package_arch = machine.raw_meta.arch orelse machine.backend.allocator.dupe(u8, "") catch return machine.stateFailed(BackendError.OutOfMemory);
    machine.raw_meta.arch = null;
    errdefer machine.backend.allocator.free(package_arch);

    const package_description = machine.raw_meta.description orelse machine.backend.allocator.dupe(u8, "") catch return machine.stateFailed(BackendError.OutOfMemory);
    machine.raw_meta.description = null;
    errdefer machine.backend.allocator.free(package_description);

    const package_url = machine.raw_meta.url orelse machine.backend.allocator.dupe(u8, "") catch return machine.stateFailed(BackendError.OutOfMemory);
    machine.raw_meta.url = null;
    errdefer machine.backend.allocator.free(package_url);

    const package_packager = machine.raw_meta.packager orelse machine.backend.allocator.dupe(u8, "") catch return machine.stateFailed(BackendError.OutOfMemory);
    machine.raw_meta.packager = null;
    errdefer machine.backend.allocator.free(package_packager);

    const package_author = machine.backend.allocator.dupe(u8, package_packager) catch return machine.stateFailed(BackendError.OutOfMemory);
    errdefer machine.backend.allocator.free(package_author);

    const package_license = machine.raw_meta.license orelse machine.backend.allocator.dupe(u8, "") catch return machine.stateFailed(BackendError.OutOfMemory);
    machine.raw_meta.license = null;
    errdefer machine.backend.allocator.free(package_license);

    machine.backend.meta = PackageMeta{
        .name = package_name,
        .version = parsed_version,
        .arch = package_arch,
        .author = package_author,
        .description = package_description,
        .license = package_license,
        .url = package_url,
        .packager = package_packager,
        .checksum = sha256,
        .size = machine.raw_meta.size,
        .installed_at = @intCast(@divTrunc(std.Io.Clock.real.now(machine.backend.io).nanoseconds, std.time.ns_per_s)),
    };

    machine.raw_meta.deinit(machine.backend.allocator);

    return .done;
}
