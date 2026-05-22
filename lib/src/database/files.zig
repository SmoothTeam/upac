const std = @import("std");
const lmdbx = @import("lmdbx");
const serde = @import("serde");

const types = @import("upac-types");
const FileEntry = types.FileEntry;

const database = @import("database.zig");
const Database = database.Database;
const DatabaseError = database.DatabaseError;

fn idToKey(allocator: std.mem.Allocator, package_id: u64, file_path: []const u8) std.mem.Allocator.Error![]u8 {
    const key = try allocator.alloc(u8, 8 + file_path.len);
    std.mem.writeInt(u64, key[0..8], package_id, .big);
    @memcpy(key[8..], file_path);
    return key;
}

fn packageIdPrefix(package_id: u64) [8]u8 {
    var prefix: [8]u8 = undefined;
    std.mem.writeInt(u64, &prefix, package_id, .big);
    return prefix;
}

pub fn insert(base: Database, package_id: u64, file_entry: FileEntry) DatabaseError!void {
    const files_base = base.files_dbi orelse return DatabaseError.PackageNotFound;
    const key = idToKey(base.allocator, package_id, file_entry.path) catch return DatabaseError.AllocZFailed;
    defer base.allocator.free(key);

    if (files_base.get(key) catch return DatabaseError.ReadError) |existing_bytes| {
        const existing_entry = serde.deserialize(FileEntry, base.allocator, existing_bytes) catch return DatabaseError.ReadError;
        defer existing_entry.deinit(base.allocator);

        if (existing_entry.is_user) return;
    }

    const file_entry_as_bytes = serde.serialize(base.allocator, file_entry) catch return DatabaseError.WriteError;
    defer base.allocator.free(file_entry_as_bytes);

    files_base.put(key, file_entry_as_bytes, .{}) catch return DatabaseError.WriteError;
}

pub fn delete(base: Database, package_id: u64, file_path: []const u8) DatabaseError!void {
    const files_base = base.files_dbi orelse return DatabaseError.PackageNotFound;
    const key = idToKey(base.allocator, package_id, file_path) catch return DatabaseError.AllocZFailed;
    defer base.allocator.free(key);

    if (files_base.get(key) catch return DatabaseError.ReadError) |existing_bytes| {
        const existing_entry = serde.deserialize(FileEntry, base.allocator, existing_bytes) catch return DatabaseError.ReadError;
        defer existing_entry.deinit(base.allocator);

        if (existing_entry.is_user) return;
    }

    files_base.del(key, null) catch return DatabaseError.WriteError;
}

pub fn update(base: Database, package_id: u64, file_entry: FileEntry) DatabaseError!void {
    const files_base = base.files_dbi orelse return DatabaseError.PackageNotFound;
    const key = idToKey(base.allocator, package_id, file_entry.path) catch return DatabaseError.AllocZFailed;
    defer base.allocator.free(key);

    if (files_base.get(key) catch return DatabaseError.ReadError) |existing_bytes| {
        const existing_entry = serde.deserialize(FileEntry, base.allocator, existing_bytes) catch return DatabaseError.ReadError;
        defer existing_entry.deinit(base.allocator);

        if (existing_entry.is_user) return;
    }

    const file_entry_as_bytes = serde.serialize(base.allocator, file_entry) catch return DatabaseError.WriteError;
    defer base.allocator.free(file_entry_as_bytes);

    files_base.put(key, file_entry_as_bytes, .{}) catch return DatabaseError.WriteError;
}

pub fn exists(base: Database, package_id: u64, file_path: []const u8) DatabaseError!bool {
    const files_base = base.files_dbi orelse return DatabaseError.PackageNotFound;
    const key = idToKey(base.allocator, package_id, file_path) catch return DatabaseError.AllocZFailed;
    defer base.allocator.free(key);

    const result = files_base.get(key) catch return DatabaseError.ReadError;
    return result != null;
}

pub fn list(base: Database, package_id: u64) DatabaseError![]FileEntry {
    const files_base = base.files_dbi orelse return DatabaseError.PackageNotFound;
    const package_id_prefix = packageIdPrefix(package_id);

    var cursor = files_base.openCursor() catch return DatabaseError.ReadError;
    defer cursor.close();

    var file_entries_list = std.ArrayList(FileEntry).init(base.allocator);
    errdefer {
        for (file_entries_list.items) |*file_entry| file_entry.deinit(base.allocator);
        file_entries_list.deinit();
    }

    var current_entry = cursor.seek(&package_id_prefix) catch return DatabaseError.ReadError;
    while (current_entry) |key_value_pair| {
        if (!std.mem.startsWith(u8, key_value_pair.key, &package_id_prefix)) break;

        const file_entry = serde.deserialize(FileEntry, base.allocator, key_value_pair.value) catch return DatabaseError.ReadError;
        file_entries_list.append(file_entry) catch return DatabaseError.AllocZFailed;

        current_entry = cursor.next() catch return DatabaseError.ReadError;
    }

    return file_entries_list.toOwnedSlice() catch return DatabaseError.AllocZFailed;
}
