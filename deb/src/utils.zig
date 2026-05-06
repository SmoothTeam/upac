// ── Imports ─────────────────────────────────────────────────────────────────────
const backend = @import("backend.zig");
const std = backend.std;
const c_libs = backend.c_libs;

const BackendMachine = backend.BackendMachine;
const BackendError = backend.BackendError;

const stateFailed = backend.stateFailed;

pub fn parseLicenseFromCopyright(content: ?[]const u8, allocator: std.mem.Allocator) BackendError![]const u8 {
    const text = content orelse return allocator.dupe(u8, "Unknown") catch BackendError.AllocZFailed;

    var lines = std.mem.splitScalar(u8, text, '\n');
    while (lines.next()) |line| {
        const trimmed_line = std.mem.trim(u8, line, " \t\r");
        if (!std.mem.startsWith(u8, trimmed_line, "License:")) continue;

        const value = std.mem.trim(u8, trimmed_line["License:".len..], " \t\r");
        if (value.len == 0) continue;

        return allocator.dupe(u8, value) catch BackendError.AllocZFailed;
    }

    return allocator.dupe(u8, "Unknown") catch BackendError.AllocZFailed;
}

pub fn copyArchiveEntry(reader: *c_libs.archive, writer: *c_libs.archive, machine: *BackendMachine) BackendError!void {
    while (true) {
        if (machine.isCancelRequested()) {
            stateFailed(machine);
            return BackendError.Cancelled;
        }
        var block: ?*const anyopaque = null;
        var block_size: usize = 0;
        var offset: i64 = 0;

        const rd = c_libs.archive_read_data_block(reader, &block, &block_size, &offset);
        if (rd == c_libs.ARCHIVE_EOF) break;
        if (rd != c_libs.ARCHIVE_OK) {
            stateFailed(machine);
            return BackendError.ArchiveReadFailed;
        }
        if (c_libs.archive_write_data_block(writer, block, block_size, offset) != c_libs.ARCHIVE_OK) {
            stateFailed(machine);
            return BackendError.ArchiveExtractFailed;
        }
    }
}

pub fn computeMd5(io: std.Io, file: std.Io.File, buf: []u8) !([std.crypto.hash.Md5.digest_length]u8) {
    var hasher = std.crypto.hash.Md5.init(.{});
    while (true) {
        const iov = [1][]u8{buf};
        const n = file.readStreaming(io, &iov) catch |err| {
            if (err == error.EndOfStream) break;
            return err;
        };
        if (n == 0) break;
        hasher.update(buf[0..n]);
    }
    var digest: [std.crypto.hash.Md5.digest_length]u8 = undefined;
    hasher.final(&digest);
    return digest;
}

pub fn isControlFile(file_name: []const u8) bool {
    const name = if (std.mem.startsWith(u8, file_name, "./")) file_name[2..] else file_name;
    return std.mem.eql(u8, name, "control");
}

pub fn isCopyrightFile(file_name: []const u8) bool {
    const name = if (std.mem.startsWith(u8, file_name, "./")) file_name[2..] else file_name;
    return std.mem.startsWith(u8, name, "usr/share/doc/") and std.mem.endsWith(u8, name, "/copyright");
}

pub fn readFileFromNestedTar(machine: *BackendMachine, outer_reader: *c_libs.archive, current_entry: ?*c_libs.archive_entry, comptime predicate: fn (name: []const u8) bool) BackendError!?[]u8 {
    const size: usize = @intCast(c_libs.archive_entry_size(current_entry));
    if (size == 0) return null;

    const tar_buf = try machine.check(machine.allocator.alloc(u8, size), BackendError.OutOfMemory);
    defer machine.allocator.free(tar_buf);

    if (c_libs.archive_read_data(outer_reader, tar_buf.ptr, size) < 0)
        return BackendError.ArchiveReadFailed;

    const inner = try machine.unwrap(c_libs.archive_read_new(), BackendError.ArchiveOpenFailed);
    defer _ = c_libs.archive_read_free(inner);

    _ = c_libs.archive_read_support_format_tar(inner);
    _ = c_libs.archive_read_support_filter_all(inner);

    if (c_libs.archive_read_open_memory(inner, tar_buf.ptr, size) != c_libs.ARCHIVE_OK)
        return BackendError.ArchiveOpenFailed;

    var inner_entry: ?*c_libs.archive_entry = null;
    while (c_libs.archive_read_next_header(inner, &inner_entry) == c_libs.ARCHIVE_OK) {
        const name = std.mem.span(c_libs.archive_entry_pathname(inner_entry));
        if (!predicate(name)) continue;

        const content_size: usize = @intCast(c_libs.archive_entry_size(inner_entry));
        const content = try machine.check(machine.allocator.alloc(u8, content_size), BackendError.OutOfMemory);
        errdefer machine.allocator.free(content);

        if (c_libs.archive_read_data(inner, content.ptr, content_size) < 0)
            return BackendError.ArchiveReadFailed;

        return content;
    }

    return null;
}
