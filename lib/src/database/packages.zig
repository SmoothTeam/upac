const std = @import("std");
const lmdbx = @import("lmdbx");
const serde = @import("serde");

const types = @import("upac-types");
const PackageMeta = types.PackageMeta;

const database = @import("database.zig");
const Database = database.Database;
const DatabaseError = database.DatabaseError;

fn idToKey(package_id: u64) [8]u8 {
    var key: [8]u8 = undefined;
    std.mem.writeInt(u64, &key, package_id, .big);
    return key;
}

pub fn insert(base: Database, meta: PackageMeta) DatabaseError!void {
    const key = idToKey(meta.id);
    const package_base = base.packages_dbi orelse return DatabaseError.PackageNotFound;

    if ((package_base.get(meta.id, &key) catch return DatabaseError.ReadError) != null) return DatabaseError.PackageAlreadyExists;

    const meta_as_bytes = serde.serialize(base.allocator, meta) catch return DatabaseError.WriteError;
    defer base.allocator.free(meta_as_bytes);

    package_base.put(&key, meta_as_bytes, .{}) catch return DatabaseError.WriteError;
}

pub fn delete(base: Database, package_id: u64) DatabaseError!void {
    const key = idToKey(package_id);
    const package_base = base.packages_dbi orelse return DatabaseError.PackageNotFound;

    if ((package_base.get(package_id, &key) catch return DatabaseError.ReadError) == null) return DatabaseError.PackageNotFound;

    package_base.del(&key, null) catch return DatabaseError.WriteError;
}

pub fn update(base: Database, meta: PackageMeta) DatabaseError!void {
    const key = idToKey(meta.id);
    const package_base = base.packages_dbi orelse return DatabaseError.PackageNotFound;

    if ((package_base.get(meta.id, &key) catch return DatabaseError.ReadError) == null) return DatabaseError.PackageNotFound;

    const meta_as_bytes = serde.serialize(base.allocator, meta) catch return DatabaseError.WriteError;
    defer base.allocator.free(meta_as_bytes);

    package_base.put(&key, meta_as_bytes, .{}) catch return DatabaseError.WriteError;
}

pub fn exists(base: Database, package_id: u64) DatabaseError!bool {
    const key = idToKey(package_id);
    const package_base = base.packages_dbi orelse return DatabaseError.PackageNotFound;
    const result = package_base.get(package_id, &key) catch return DatabaseError.ReadError;
    return result != null;
}

pub fn get(base: Database, package_id: u64) DatabaseError!?PackageMeta {
    const key = idToKey(package_id);
    const package_base = base.packages_dbi orelse return DatabaseError.PackageNotFound;
    const meta_as_bytes = (package_base.get(package_id, &key) catch return DatabaseError.ReadError) orelse return null;
    return serde.deserialize(PackageMeta, base.allocator, meta_as_bytes) catch return DatabaseError.ReadError;
}

pub fn list(base: Database) DatabaseError![]PackageMeta {
    const package_base = base.packages_dbi orelse return DatabaseError.PackageNotFound;

    var cursor = package_base.openCursor() catch return DatabaseError.ReadError;
    defer cursor.close();

    var list_packages_metas = std.ArrayList(PackageMeta).init(base.allocator);
    errdefer {
        for (list_packages_metas.items) |*item| item.deinit(base.allocator);
        list_packages_metas.deinit();
    }

    var current_entry = cursor.first() catch return DatabaseError.ReadError;
    while (current_entry) |package_meta_bytes| {
        const package_meta = serde.deserialize(PackageMeta, base.allocator, package_meta_bytes) catch return DatabaseError.ReadError;
        list_packages_metas.append(package_meta) catch return DatabaseError.AllocZFailed;
        current_entry = cursor.next() catch return DatabaseError.ReadError;
    }

    return list_packages_metas.toOwnedSlice() catch return DatabaseError.AllocZFailed;
}
