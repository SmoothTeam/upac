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
    open_outer_archive,
    find_data_tar,
    open_inner_archive,
    next_entry,
    write_blocks,
    close_archives,
    done,
};

// ── UnpackingMachine ──────────────────────────────────────────────────────────
const UnpackingMachine = struct {
    backend: *Machine,

    inner_reader: ?*c_libs.archive = null,
    outer_reader: ?*c_libs.archive = null,

    data_tar_buf: ?[]u8 = null,

    archive_writer: ?*c_libs.archive = null,

    old_package_dir: ?std.Io.Dir = null,

    fn stateFailed(self: *UnpackingMachine, err: BackendError) BackendError {
        if (self.outer_reader) |reader| {
            _ = c_libs.archive_read_free(reader);
            self.outer_reader = null;
        }

        if (self.inner_reader) |reader| {
            _ = c_libs.archive_read_free(reader);
            self.inner_reader = null;
        }

        if (self.data_tar_buf) |buf| {
            self.backend.allocator.free(buf);
            self.data_tar_buf = null;
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
            .open_outer_archive => try stateOpenOuterArchive(&unpacking),
            .find_data_tar => try stateFindDataTar(&unpacking),
            .open_inner_archive => try stateOpenInnerArchive(&unpacking),
            .next_entry => try stateNextEntry(&unpacking),
            .write_blocks => try stateWriteBlocks(&unpacking),
            .close_archives => stateCloseArchives(&unpacking),
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

    return .open_outer_archive;
}

fn stateOpenOuterArchive(machine: *UnpackingMachine) BackendError!UnpackingState {
    const temp_package_path = machine.backend.temp_package_path orelse return machine.stateFailed(BackendError.TempDirFailed);

    const outer_reader = c_libs.archive_read_new() orelse return machine.stateFailed(BackendError.ArchiveOpenFailed);
    machine.outer_reader = outer_reader;

    _ = c_libs.archive_read_support_format_ar(outer_reader);
    _ = c_libs.archive_read_support_filter_all(outer_reader);

    if (c_libs.archive_read_open_filename(outer_reader, machine.backend.data.package_path_c, 16384) != c_libs.ARCHIVE_OK) return machine.stateFailed(BackendError.ArchiveOpenFailed);

    const archive_writer = c_libs.archive_write_disk_new() orelse return machine.stateFailed(BackendError.ArchiveOpenFailed);
    machine.archive_writer = archive_writer;

    _ = c_libs.archive_write_disk_set_options(archive_writer, c_libs.ARCHIVE_EXTRACT_TIME |
        c_libs.ARCHIVE_EXTRACT_PERM |
        c_libs.ARCHIVE_EXTRACT_FFLAGS);
    _ = c_libs.archive_write_disk_set_standard_lookup(archive_writer);

    const old_package_dir = std.Io.Dir.cwd().openDir(machine.backend.io, ".", .{}) catch return machine.stateFailed(BackendError.ReadFailed);
    machine.old_package_dir = old_package_dir;

    std.Io.Threaded.chdir(temp_package_path) catch return machine.stateFailed(BackendError.TempDirFailed);

    return .find_data_tar;
}

fn stateFindDataTar(machine: *UnpackingMachine) BackendError!UnpackingState {
    const outer_reader = machine.outer_reader orelse return machine.stateFailed(BackendError.ArchiveOpenFailed);

    var outer_entry: ?*c_libs.archive_entry = null;
    const result = c_libs.archive_read_next_header(outer_reader, &outer_entry);

    if (result == c_libs.ARCHIVE_EOF) return machine.stateFailed(BackendError.InvalidPackage);
    if (result != c_libs.ARCHIVE_OK) return machine.stateFailed(BackendError.ArchiveReadFailed);

    const entry_name = std.mem.span(c_libs.archive_entry_pathname(outer_entry));
    if (!std.mem.startsWith(u8, entry_name, "data.tar")) return .find_data_tar;

    const raw_size = c_libs.archive_entry_size(outer_entry);
    if (raw_size <= 0) return machine.stateFailed(BackendError.InvalidPackage);

    const data_size: usize = @intCast(raw_size);
    const data_buf = machine.backend.allocator.alloc(u8, data_size) catch return machine.stateFailed(BackendError.OutOfMemory);
    machine.data_tar_buf = data_buf;

    if (c_libs.archive_read_data(outer_reader, data_buf.ptr, data_size) < 0) return machine.stateFailed(BackendError.ArchiveReadFailed);

    return .open_inner_archive;
}

fn stateOpenInnerArchive(machine: *UnpackingMachine) BackendError!UnpackingState {
    const data_buf = machine.data_tar_buf orelse return machine.stateFailed(BackendError.InvalidPackage);

    if (machine.outer_reader) |reader| {
        _ = c_libs.archive_read_free(reader);
        machine.outer_reader = null;
    }

    const inner_reader = c_libs.archive_read_new() orelse return machine.stateFailed(BackendError.ArchiveOpenFailed);
    machine.inner_reader = inner_reader;

    _ = c_libs.archive_read_support_format_tar(inner_reader);
    _ = c_libs.archive_read_support_filter_all(inner_reader);

    if (c_libs.archive_read_open_memory(inner_reader, data_buf.ptr, data_buf.len) != c_libs.ARCHIVE_OK) return machine.stateFailed(BackendError.ArchiveOpenFailed);

    return .next_entry;
}

fn stateNextEntry(machine: *UnpackingMachine) BackendError!UnpackingState {
    var archive_entry: ?*c_libs.archive_entry = undefined;

    const inner_reader = machine.inner_reader orelse return machine.stateFailed(BackendError.ArchiveOpenFailed);
    const archive_writer = machine.archive_writer orelse return machine.stateFailed(BackendError.ArchiveOpenFailed);

    const read_result = c_libs.archive_read_next_header(inner_reader, &archive_entry);

    if (read_result == c_libs.ARCHIVE_EOF) return .close_archives;
    if (read_result != c_libs.ARCHIVE_OK) return machine.stateFailed(BackendError.ArchiveReadFailed);

    const entry = archive_entry orelse return machine.stateFailed(BackendError.ArchiveReadFailed);

    if (c_libs.archive_write_header(archive_writer, entry) != c_libs.ARCHIVE_OK) return machine.stateFailed(BackendError.ArchiveExtractFailed);

    return .write_blocks;
}

fn stateWriteBlocks(machine: *UnpackingMachine) BackendError!UnpackingState {
    var block_size: usize = 0;
    var block_offset: i64 = 0;
    var data_block: ?*const anyopaque = null;

    const inner_reader = machine.inner_reader orelse return machine.stateFailed(BackendError.ArchiveOpenFailed);
    const archive_writer = machine.archive_writer orelse return machine.stateFailed(BackendError.ArchiveOpenFailed);

    const block_result = c_libs.archive_read_data_block(inner_reader, &data_block, &block_size, &block_offset);
    if (block_result == c_libs.ARCHIVE_EOF) {
        if (c_libs.archive_write_finish_entry(archive_writer) != c_libs.ARCHIVE_OK) return machine.stateFailed(BackendError.ArchiveExtractFailed);

        return .next_entry;
    }
    if (block_result != c_libs.ARCHIVE_OK) return machine.stateFailed(BackendError.ArchiveReadFailed);

    if (c_libs.archive_write_data_block(archive_writer, data_block, block_size, block_offset) != c_libs.ARCHIVE_OK) return machine.stateFailed(BackendError.ArchiveExtractFailed);

    return .write_blocks;
}

fn stateCloseArchives(machine: *UnpackingMachine) UnpackingState {
    if (machine.inner_reader) |reader| {
        _ = c_libs.archive_read_free(reader);
        machine.inner_reader = null;
    }

    if (machine.outer_reader) |reader| {
        _ = c_libs.archive_read_free(reader);
        machine.outer_reader = null;
    }

    if (machine.data_tar_buf) |buf| {
        machine.backend.allocator.free(buf);
        machine.data_tar_buf = null;
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
