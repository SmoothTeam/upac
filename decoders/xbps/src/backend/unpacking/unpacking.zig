// SPDX-FileCopyrightText: 2026 2026 JustPav
// SPDX-FileCopyrightText: 2026 JustPav
//
// SPDX-License-Identifier: LGPL-3.0-or-later

// ── Imports ───────────────────────────────────────────────────────────────────
const std = @import("std");

const c_libs = @import("c-libs");

const types = @import("upac-backend-types");
const BackendError = types.BackendError;

const backend = @import("../backend.zig");
const Machine = backend.BackendMachine;

// ── UnpackingState ────────────────────────────────────────────────────────────
const UnpackingState = enum {
    create_temp_dir,
    open_archive,
    next_entry,
    write_blocks,
    close_archive,
    done,
};

// ── UnpackingMachine ──────────────────────────────────────────────────────────
const UnpackingMachine = struct {
    backend: *Machine,

    archive_reader: ?*c_libs.archive = null,
    archive_writer: ?*c_libs.archive = null,

    old_package_dir: ?std.Io.Dir = null,

    fn stateFailed(self: *UnpackingMachine, err: BackendError) BackendError {
        if (self.archive_reader) |reader| {
            _ = c_libs.archive_read_free(reader);
            self.archive_reader = null;
        }

        if (self.archive_writer) |writer| {
            _ = c_libs.archive_write_free(writer);
            self.archive_writer = null;
        }

        if (self.old_package_dir) |old_dir| {
            std.Io.Threaded.fchdir(old_dir.handle) catch {};
            old_dir.close(self.backend.io);
            self.old_package_dir = null;
        }

        if (self.backend.props_content) |content| {
            self.backend.allocator.free(content);
            self.backend.props_content = null;
        }

        if (self.backend.temp_package_path) |temp_path| {
            std.Io.Dir.cwd().deleteTree(self.backend.io, temp_path) catch {};
            self.backend.allocator.free(temp_path);
            self.backend.temp_package_path = null;
        }

        return err;
    }
};

// ── Trampoline ────────────────────────────────────────────────────────────────
pub fn run(machine: *Machine) BackendError!void {
    var unpacking = UnpackingMachine{ .backend = machine };

    var state = UnpackingState.create_temp_dir;
    while (state != .done) {
        if (machine.data.cancel_token.isCancelled()) return unpacking.stateFailed(BackendError.Cancelled);
        state = switch (state) {
            .create_temp_dir => try stateCreateTempDir(&unpacking),
            .open_archive => try stateOpenArchive(&unpacking),
            .next_entry => try stateNextEntry(&unpacking),
            .write_blocks => try stateWriteBlocks(&unpacking),
            .close_archive => stateCloseArchive(&unpacking),
            .done => unreachable,
        };
    }
}

// ── States ────────────────────────────────────────────────────────────────────
fn stateCreateTempDir(machine: *UnpackingMachine) BackendError!UnpackingState {
    var temp_dir_name_buf: [256]u8 = undefined;

    const temp_path = std.mem.span(machine.backend.data.temp_path_c);
    const timestamp: i64 = @intCast(@divTrunc(std.Io.Clock.real.now(machine.backend.io).nanoseconds, std.time.ns_per_ms));

    const temp_package_dir_name = std.fmt.bufPrintZ(&temp_dir_name_buf, "upac-installed-{d}", .{timestamp}) catch return machine.stateFailed(BackendError.AllocZFailed);

    const temp_package_path = std.Io.Dir.path.joinZ(machine.backend.allocator, &.{ temp_path, temp_package_dir_name }) catch return machine.stateFailed(BackendError.AllocZFailed);

    std.Io.Dir.createDirAbsolute(machine.backend.io, temp_package_path, .default_dir) catch return machine.stateFailed(BackendError.TempDirFailed);

    machine.backend.temp_package_path = temp_package_path;

    return .open_archive;
}

fn stateOpenArchive(machine: *UnpackingMachine) BackendError!UnpackingState {
    const temp_package_path = machine.backend.temp_package_path orelse return machine.stateFailed(BackendError.TempDirFailed);

    const archive_reader = c_libs.archive_read_new() orelse return machine.stateFailed(BackendError.ArchiveOpenFailed);
    machine.archive_reader = archive_reader;

    _ = c_libs.archive_read_support_format_tar(archive_reader);
    _ = c_libs.archive_read_support_filter_zstd(archive_reader);
    _ = c_libs.archive_read_support_filter_xz(archive_reader);
    _ = c_libs.archive_read_support_filter_gzip(archive_reader);

    if (c_libs.archive_read_open_filename(archive_reader, machine.backend.data.package_path_c, 16384) != c_libs.ARCHIVE_OK) return machine.stateFailed(BackendError.ArchiveOpenFailed);

    const archive_writer = c_libs.archive_write_disk_new() orelse return machine.stateFailed(BackendError.ArchiveOpenFailed);
    machine.archive_writer = archive_writer;

    _ = c_libs.archive_write_disk_set_options(archive_writer, c_libs.ARCHIVE_EXTRACT_TIME |
        c_libs.ARCHIVE_EXTRACT_PERM |
        c_libs.ARCHIVE_EXTRACT_FFLAGS);
    _ = c_libs.archive_write_disk_set_standard_lookup(archive_writer);

    const old_package_dir = std.Io.Dir.cwd().openDir(machine.backend.io, ".", .{}) catch return machine.stateFailed(BackendError.ReadFailed);
    machine.old_package_dir = old_package_dir;

    std.Io.Threaded.chdir(temp_package_path) catch return machine.stateFailed(BackendError.TempDirFailed);

    return .next_entry;
}

fn stateNextEntry(machine: *UnpackingMachine) BackendError!UnpackingState {
    const archive_reader = machine.archive_reader orelse return machine.stateFailed(BackendError.ArchiveOpenFailed);
    const archive_writer = machine.archive_writer orelse return machine.stateFailed(BackendError.ArchiveOpenFailed);

    var archive_entry: ?*c_libs.archive_entry = null;
    const read_result = c_libs.archive_read_next_header(archive_reader, &archive_entry);

    if (read_result == c_libs.ARCHIVE_EOF) return .close_archive;
    if (read_result != c_libs.ARCHIVE_OK) return machine.stateFailed(BackendError.ArchiveReadFailed);

    const entry = archive_entry orelse return machine.stateFailed(BackendError.ArchiveReadFailed);

    const entry_path = c_libs.archive_entry_pathname(entry);
    const entry_name_raw = if (entry_path != null) std.mem.span(entry_path) else "";
    const entry_name = if (std.mem.startsWith(u8, entry_name_raw, "./")) entry_name_raw[2..] else entry_name_raw;

    if (std.mem.eql(u8, entry_name, "props.plist")) {
        const raw_size = c_libs.archive_entry_size(entry);
        if (raw_size <= 0) return .next_entry;

        const entry_size: usize = @intCast(raw_size);
        const content_buf = machine.backend.allocator.alloc(u8, entry_size) catch return machine.stateFailed(BackendError.OutOfMemory);
        machine.backend.props_content = content_buf;

        if (c_libs.archive_read_data(archive_reader, content_buf.ptr, entry_size) < 0) return machine.stateFailed(BackendError.ArchiveReadFailed);

        return .next_entry;
    }

    if (std.mem.eql(u8, entry_name, "files.plist") or
        std.mem.eql(u8, entry_name, "INSTALL") or
        std.mem.eql(u8, entry_name, "REMOVE"))
    {
        _ = c_libs.archive_read_data_skip(archive_reader);
        return .next_entry;
    }

    if (c_libs.archive_write_header(archive_writer, entry) != c_libs.ARCHIVE_OK) return machine.stateFailed(BackendError.ArchiveExtractFailed);

    return .write_blocks;
}

fn stateWriteBlocks(machine: *UnpackingMachine) BackendError!UnpackingState {
    var block_size: usize = 0;
    var block_offset: i64 = 0;
    var data_block: ?*const anyopaque = null;

    const archive_reader = machine.archive_reader orelse return machine.stateFailed(BackendError.ArchiveOpenFailed);
    const archive_writer = machine.archive_writer orelse return machine.stateFailed(BackendError.ArchiveOpenFailed);

    const block_result = c_libs.archive_read_data_block(archive_reader, &data_block, &block_size, &block_offset);
    if (block_result == c_libs.ARCHIVE_EOF) {
        if (c_libs.archive_write_finish_entry(archive_writer) != c_libs.ARCHIVE_OK) return machine.stateFailed(BackendError.ArchiveExtractFailed);

        return .next_entry;
    }
    if (block_result != c_libs.ARCHIVE_OK) return machine.stateFailed(BackendError.ArchiveReadFailed);

    if (c_libs.archive_write_data_block(archive_writer, data_block, block_size, block_offset) != c_libs.ARCHIVE_OK) return machine.stateFailed(BackendError.ArchiveExtractFailed);

    return .write_blocks;
}

fn stateCloseArchive(machine: *UnpackingMachine) UnpackingState {
    if (machine.archive_reader) |reader| {
        _ = c_libs.archive_read_free(reader);
        machine.archive_reader = null;
    }

    if (machine.archive_writer) |writer| {
        _ = c_libs.archive_write_free(writer);
        machine.archive_writer = null;
    }

    if (machine.old_package_dir) |old_dir| {
        std.Io.Threaded.fchdir(old_dir.handle) catch {};
        old_dir.close(machine.backend.io);
        machine.old_package_dir = null;
    }

    return .done;
}
