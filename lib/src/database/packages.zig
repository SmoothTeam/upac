const std = @import("std");
const lmdbx = @import("lmdbx");
const serde = @import("serde");

const types = @import("upac-types");
const PackageMeta = types.PackageMeta;

const database = @import("database.zig");
const Database = database.Database;
const DatabaseError = database.DatabaseError;

pub fn insert(base: Database, meta: PackageMeta) DatabaseError![16]u8 {
    const package_base = base.packages_dbi orelse return DatabaseError.PackageNotFound;

    var uuid: [16]u8 = undefined;
    _ = std.os.linux.getrandom(&uuid, uuid.len, 0);

    const meta_as_bytes = serde.msgpack.toSlice(base.allocator, meta) catch return DatabaseError.WriteError;
    defer base.allocator.free(meta_as_bytes);

    package_base.set(&uuid, meta_as_bytes, .Upsert) catch return DatabaseError.WriteError;

    return uuid;
}

pub fn delete(base: Database, name: []const u8, arch: []const u8, arch_sub: ?[]const u8) DatabaseError!void {
    const package_base = base.packages_dbi orelse return DatabaseError.PackageNotFound;
    const found = try exists(base, name, arch, arch_sub) orelse return DatabaseError.PackageNotFound;

    package_base.delete(&found) catch return DatabaseError.WriteError;
}

pub fn update(base: Database, meta: PackageMeta) DatabaseError!void {
    const package_base = base.packages_dbi orelse return DatabaseError.PackageNotFound;
    const found = try exists(base, meta.name, meta.arch, meta.arch_sub) orelse return DatabaseError.PackageNotFound;

    const meta_as_bytes = serde.msgpack.toSlice(base.allocator, meta) catch return DatabaseError.WriteError;
    defer base.allocator.free(meta_as_bytes);

    package_base.set(&found, meta_as_bytes, .Upsert) catch return DatabaseError.WriteError;
}

// Returns the UUID of the installed package, or null if not found.
pub fn exists(base: Database, name: []const u8, arch: []const u8, arch_sub: ?[]const u8) DatabaseError!?[16]u8 {
    const package_base = base.packages_dbi orelse return DatabaseError.PackageNotFound;

    var cursor = package_base.cursor() catch return DatabaseError.ReadError;
    defer cursor.deinit();

    var has_value = cursor.goToFirst() catch return DatabaseError.ReadError;
    while (has_value != null) {
        const entry = cursor.getCurrentEntry() catch return DatabaseError.ReadError;
        var meta = serde.msgpack.fromSlice(PackageMeta, base.allocator, entry.value) catch return DatabaseError.ReadError;
        defer meta.deinit(base.allocator);

        if (matchesIdentity(meta, name, arch, arch_sub)) {
            var key: [16]u8 = undefined;
            @memcpy(&key, entry.key[0..16]);
            return key;
        }

        has_value = cursor.goToNext() catch return DatabaseError.ReadError;
    }

    return null;
}

pub fn list(base: Database) DatabaseError![]PackageMeta {
    const package_base = base.packages_dbi orelse return DatabaseError.PackageNotFound;

    var cursor = package_base.cursor() catch return DatabaseError.ReadError;
    defer cursor.deinit();

    var list_packages_metas = std.ArrayList(PackageMeta).empty;
    errdefer {
        for (list_packages_metas.items) |*item| item.deinit(base.allocator);
        list_packages_metas.deinit(base.allocator);
    }

    var has_next = cursor.goToFirst() catch return DatabaseError.ReadError;
    while (has_next != null) {
        const entry = cursor.getCurrentEntry() catch return DatabaseError.ReadError;
        const package_meta = serde.msgpack.fromSlice(PackageMeta, base.allocator, entry.value) catch return DatabaseError.ReadError;
        list_packages_metas.append(base.allocator, package_meta) catch return DatabaseError.AllocZFailed;
        has_next = cursor.goToNext() catch return DatabaseError.ReadError;
    }

    return list_packages_metas.toOwnedSlice(base.allocator) catch return DatabaseError.AllocZFailed;
}

fn matchesIdentity(meta: PackageMeta, name: []const u8, arch: []const u8, arch_sub: ?[]const u8) bool {
    if (!std.mem.eql(u8, meta.name, name)) return false;
    if (!std.mem.eql(u8, meta.arch, arch)) return false;
    if (arch_sub == null and meta.arch_sub == null) return true;

    const source_arch_sub = arch_sub orelse return false;
    const found_arch_sub = meta.arch_sub orelse return false;

    return std.mem.eql(u8, source_arch_sub, found_arch_sub);
}
