const std = @import("std");
const lmdbx = @import("lmdbx");
const serde = @import("serde");

const types = @import("upac-types");
const FileEntry = types.FileEntry;

const database = @import("database.zig");
const Database = database.Database;
const DatabaseError = database.DatabaseError;

fn buildKey(allocator: std.mem.Allocator, uuid: [16]u8, file_path: []const u8) std.mem.Allocator.Error![]u8 {
    const key = try allocator.alloc(u8, 16 + file_path.len);
    @memcpy(key[0..16], &uuid);
    @memcpy(key[16..], file_path);
    return key;
}

pub fn insert(base: Database, uuid: [16]u8, file_entry: FileEntry) DatabaseError!void {
    const files_base = base.files_dbi orelse return DatabaseError.PackageNotFound;
    const key = buildKey(base.allocator, uuid, file_entry.path) catch return DatabaseError.AllocZFailed;
    defer base.allocator.free(key);

    if (files_base.get(key) catch return DatabaseError.ReadError) |existing_bytes| {
        var existing_entry = serde.msgpack.fromSlice(FileEntry, base.allocator, existing_bytes) catch return DatabaseError.ReadError;
        defer existing_entry.deinit(base.allocator);

        if (existing_entry.is_user) return;
    }

    const file_entry_as_bytes = serde.msgpack.toSlice(base.allocator, file_entry) catch return DatabaseError.WriteError;
    defer base.allocator.free(file_entry_as_bytes);

    files_base.set(key, file_entry_as_bytes, .Upsert) catch return DatabaseError.WriteError;
}

pub fn delete(base: Database, uuid: [16]u8, file_path: []const u8) DatabaseError!void {
    const files_base = base.files_dbi orelse return DatabaseError.PackageNotFound;
    const key = buildKey(base.allocator, uuid, file_path) catch return DatabaseError.AllocZFailed;
    defer base.allocator.free(key);

    if (files_base.get(key) catch return DatabaseError.ReadError) |existing_bytes| {
        const existing_entry = serde.msgpack.fromSlice(FileEntry, base.allocator, existing_bytes) catch return DatabaseError.ReadError;
        defer existing_entry.deinit(base.allocator);

        if (existing_entry.is_user) return;
    }

    files_base.delete(key) catch return DatabaseError.WriteError;
}

pub fn update(base: Database, uuid: [16]u8, file_entry: FileEntry) DatabaseError!void {
    const files_base = base.files_dbi orelse return DatabaseError.PackageNotFound;
    const key = buildKey(base.allocator, uuid, file_entry.path) catch return DatabaseError.AllocZFailed;
    defer base.allocator.free(key);

    if (files_base.get(key) catch return DatabaseError.ReadError) |existing_bytes| {
        const existing_entry = serde.msgpack.fromSlice(FileEntry, base.allocator, existing_bytes) catch return DatabaseError.ReadError;
        defer existing_entry.deinit(base.allocator);

        if (existing_entry.is_user) return;
    }

    const file_entry_as_bytes = serde.msgpack.toSlice(base.allocator, file_entry) catch return DatabaseError.WriteError;
    defer base.allocator.free(file_entry_as_bytes);

    files_base.set(key, file_entry_as_bytes, .Upsert) catch return DatabaseError.WriteError;
}

pub fn exists(base: Database, uuid: [16]u8, file_path: []const u8) DatabaseError!bool {
    const files_base = base.files_dbi orelse return DatabaseError.PackageNotFound;
    const key = buildKey(base.allocator, uuid, file_path) catch return DatabaseError.AllocZFailed;
    defer base.allocator.free(key);

    const result = files_base.get(key) catch return DatabaseError.ReadError;
    return result != null;
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

    const first = cursor.seekLowerBound(&uuid) catch return DatabaseError.ReadError;
    var current_entry: ?lmdbx.Cursor.Entry = if (first) |result| result.entry else null;
    while (current_entry) |key_value_pair| {
        if (!std.mem.startsWith(u8, key_value_pair.key, &uuid)) break;

        const file_entry = serde.msgpack.fromSlice(FileEntry, base.allocator, key_value_pair.value) catch return DatabaseError.ReadError;
        file_entries_list.append(base.allocator, file_entry) catch return DatabaseError.AllocZFailed;

        const has_next = cursor.goToNext() catch return DatabaseError.ReadError;
        current_entry = if (has_next != null) cursor.getCurrentEntry() catch return DatabaseError.ReadError else null;
    }

    return file_entries_list.toOwnedSlice(base.allocator) catch return DatabaseError.AllocZFailed;
}
