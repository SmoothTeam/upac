const std = @import("std");
const serde = @import("serde");
const toSlice = serde.msgpack.toSlice;
const fromSlice = serde.msgpack.fromSlice;

const types = @import("upac-types");
const FileEntry = types.FileEntry;

const database = @import("database.zig");
const Database = database.Database;
const DatabaseError = database.DatabaseError;

pub fn insert(base: Database, uuid: [16]u8, file_entry: FileEntry) DatabaseError!void {
    const files_base = base.files_dbi orelse return DatabaseError.PackageNotFound;

    var cursor = files_base.cursor() catch return DatabaseError.ReadError;
    defer cursor.deinit();

    const seek_result = cursor.seekLowerBound(&uuid) catch return DatabaseError.ReadError;
    if (seek_result) |result| {
        if (result.exact) {
            var current_value: []const u8 = result.entry.value;
            while (true) {
                var existing = fromSlice(FileEntry, base.allocator, current_value) catch return DatabaseError.ReadError;
                defer existing.deinit(base.allocator);
                if (std.mem.eql(u8, existing.path, file_entry.path)) {
                    if (existing.is_user) return;
                    cursor.del(.Current) catch return DatabaseError.WriteError;
                    break;
                }
                const next_dup = cursor.nextDup() catch return DatabaseError.ReadError;
                current_value = (next_dup orelse break).value;
            }
        }
    }

    const serialized = toSlice(base.allocator, file_entry) catch return DatabaseError.WriteError;
    defer base.allocator.free(serialized);
    cursor.put(&uuid, serialized, .Upsert) catch return DatabaseError.WriteError;
}

pub fn delete(base: Database, uuid: [16]u8, file_path: []const u8) DatabaseError!void {
    const files_base = base.files_dbi orelse return DatabaseError.PackageNotFound;

    var cursor = files_base.cursor() catch return DatabaseError.ReadError;
    defer cursor.deinit();

    const seek_result = cursor.seekLowerBound(&uuid) catch return DatabaseError.ReadError;
    const result = seek_result orelse return;
    if (!result.exact) return;

    var current_value: []const u8 = result.entry.value;
    while (true) {
        var existing = fromSlice(FileEntry, base.allocator, current_value) catch return DatabaseError.ReadError;
        defer existing.deinit(base.allocator);
        if (std.mem.eql(u8, existing.path, file_path)) {
            if (existing.is_user) return;
            cursor.del(.Current) catch return DatabaseError.WriteError;
            return;
        }
        const next_dup = cursor.nextDup() catch return DatabaseError.ReadError;
        current_value = (next_dup orelse return).value;
    }
}

pub fn update(base: Database, uuid: [16]u8, file_entry: FileEntry) DatabaseError!void {
    const files_base = base.files_dbi orelse return DatabaseError.PackageNotFound;

    var cursor = files_base.cursor() catch return DatabaseError.ReadError;
    defer cursor.deinit();

    const seek_result = cursor.seekLowerBound(&uuid) catch return DatabaseError.ReadError;
    if (seek_result) |result| {
        if (result.exact) {
            var current_value: []const u8 = result.entry.value;
            while (true) {
                var existing = fromSlice(FileEntry, base.allocator, current_value) catch return DatabaseError.ReadError;
                defer existing.deinit(base.allocator);
                if (std.mem.eql(u8, existing.path, file_entry.path)) {
                    if (existing.is_user) return;
                    cursor.del(.Current) catch return DatabaseError.WriteError;
                    break;
                }
                const next_dup = cursor.nextDup() catch return DatabaseError.ReadError;
                current_value = (next_dup orelse break).value;
            }
        }
    }

    const serialized = toSlice(base.allocator, file_entry) catch return DatabaseError.WriteError;
    defer base.allocator.free(serialized);
    cursor.put(&uuid, serialized, .Upsert) catch return DatabaseError.WriteError;
}

pub fn exists(base: Database, uuid: [16]u8, file_path: []const u8) DatabaseError!bool {
    const files_base = base.files_dbi orelse return DatabaseError.PackageNotFound;

    var cursor = files_base.cursor() catch return DatabaseError.ReadError;
    defer cursor.deinit();

    const seek_result = cursor.seekLowerBound(&uuid) catch return DatabaseError.ReadError;
    const result = seek_result orelse return false;
    if (!result.exact) return false;

    var current_value: []const u8 = result.entry.value;
    while (true) {
        var existing = fromSlice(FileEntry, base.allocator, current_value) catch return DatabaseError.ReadError;
        defer existing.deinit(base.allocator);
        if (std.mem.eql(u8, existing.path, file_path)) return true;
        const next_dup = cursor.nextDup() catch return DatabaseError.ReadError;
        current_value = (next_dup orelse return false).value;
    }
}

pub fn list(base: Database, uuid: [16]u8) DatabaseError![]FileEntry {
    const files_base = base.files_dbi orelse return DatabaseError.PackageNotFound;

    var cursor = files_base.cursor() catch return DatabaseError.ReadError;
    defer cursor.deinit();

    var file_entries_list = std.ArrayList(FileEntry).empty;
    errdefer {
        for (file_entries_list.items) |*file_entry| file_entry.deinit(base.allocator);
        file_entries_list.deinit(base.allocator);
    }

    const seek_result = cursor.seekLowerBound(&uuid) catch return DatabaseError.ReadError;
    const result = seek_result orelse return file_entries_list.toOwnedSlice(base.allocator) catch return DatabaseError.AllocZFailed;
    if (!result.exact) return file_entries_list.toOwnedSlice(base.allocator) catch return DatabaseError.AllocZFailed;

    var current_value: []const u8 = result.entry.value;
    while (true) {
        var file_entry = fromSlice(FileEntry, base.allocator, current_value) catch return DatabaseError.ReadError;
        file_entries_list.append(base.allocator, file_entry) catch {
            file_entry.deinit(base.allocator);
            return DatabaseError.AllocZFailed;
        };
        const next_dup = cursor.nextDup() catch return DatabaseError.ReadError;
        current_value = (next_dup orelse break).value;
    }

    return file_entries_list.toOwnedSlice(base.allocator) catch return DatabaseError.AllocZFailed;
}
