// ── Imports ─────────────────────────────────────────────────────────────────────
const list_module = @import("upac-list");
const std = list_module.std;
const c_libs = list_module.c_libs;
const data = list_module.data;

const CSlice = list_module.ffi.CSlice;
const CArray = list_module.ffi.CArray;
const CPackageMeta = list_module.ffi.CPackageMeta;

const CCommitEntry = list_module.ffi.CCommitEntry;

const CListRequest = list_module.ffi.CUnmutatedRequest;

const ErrorCode = list_module.ffi.ErrorCode;
const Operation = list_module.ffi.Operation;

const fromError = list_module.ffi.fromError;

pub fn list_packages(list_request_c: CListRequest, out_c: *CArray(CPackageMeta)) callconv(.c) i32 {
    const required = [_]CSlice{ list_request_c.repo_path, list_request_c.branch, list_request_c.db_path };
    for (required) |field| {
        if (field.len == 0 or field.ptr[field.len] != 0)
            return @intFromEnum(fromError(error.InvalidEntry, Operation.list));
    }

    const packages = list_module.ListMachine.runPackages(.{
        .repo_path = list_request_c.repo_path.asZ(),
        .branch = list_request_c.branch.asZ(),
        .db_path = list_request_c.db_path.toSlice(),
    }, list_module.ffi.allocator()) catch |err| {
        if (err == error.Cancelled) list_module.ffi.global_cancel.store(true, .release);
        return @intFromEnum(fromError(err, Operation.list));
    };

    out_c.* = .{ .ptr = packages.ptr, .len = packages.len };
    return @intFromEnum(ErrorCode.ok);
}

pub fn get_packages_count(out_c: *CArray(CPackageMeta)) callconv(.c) usize {
    return out_c.len;
}

pub fn get_package_at(array_c: *CArray(CPackageMeta), index: usize, out: ?*?*CPackageMeta) callconv(.c) i32 {
    const out_ptr = out orelse return @intFromEnum(fromError(error.InvalidEntry, Operation.list));
    if (index >= array_c.len) return @intFromEnum(fromError(error.InvalidEntry, Operation.list));

    out_ptr.* = &array_c.ptr[index];

    return @intFromEnum(ErrorCode.ok);
}

pub fn get_package_slice_field(out_c: *CPackageMeta, field: u8, out: ?*CSlice) callconv(.c) i32 {
    const out_ptr = out orelse return @intFromEnum(fromError(error.InvalidEntry, Operation.list));

    const result = switch (field) {
        0 => out_c.name,
        1 => out_c.version,
        2 => out_c.architecture,
        3 => out_c.author,
        4 => out_c.description,
        5 => out_c.license,
        6 => out_c.url,
        7 => out_c.packager,
        8 => out_c.checksum,
        else => return @intFromEnum(fromError(error.InvalidEntry, Operation.list)),
    };

    out_ptr.* = result;
    return @intFromEnum(ErrorCode.ok);
}

pub fn get_package_int_field(out_c: *CPackageMeta, field: u8, out: ?*u64) callconv(.c) i32 {
    const out_ptr = out orelse return @intFromEnum(fromError(error.InvalidEntry, Operation.list));

    out_ptr.* = switch (field) {
        9 => @intCast(out_c.size),
        10 => @intCast(out_c.installed_at),
        else => return @intFromEnum(fromError(error.InvalidEntry, Operation.list)),
    };

    return @intFromEnum(ErrorCode.ok);
}

pub fn packages_free(package_meta_array_c: *CArray(CPackageMeta)) callconv(.c) void {
    const allocator = list_module.ffi.allocator();
    for (package_meta_array_c.toSlice()) |package_meta_c| {
        allocator.free(package_meta_c.name.toSlice());
        allocator.free(package_meta_c.version.toSlice());
        allocator.free(package_meta_c.architecture.toSlice());
        allocator.free(package_meta_c.author.toSlice());
        allocator.free(package_meta_c.description.toSlice());
        allocator.free(package_meta_c.license.toSlice());
        allocator.free(package_meta_c.url.toSlice());
        allocator.free(package_meta_c.packager.toSlice());
        allocator.free(package_meta_c.checksum.toSlice());
    }
    allocator.free(package_meta_array_c.toSlice());
}

pub fn list_commits(list_request_c: CListRequest, out_c: *CArray(CCommitEntry)) callconv(.c) i32 {
    const required = [_]CSlice{ list_request_c.repo_path, list_request_c.branch };
    for (required) |field| {
        if (field.len == 0 or field.ptr[field.len] != 0) return @intFromEnum(fromError(error.InvalidEntry, Operation.list));
    }

    const commit_entries = list_module.ListMachine.runCommits(.{
        .repo_path = list_request_c.repo_path.asZ(),
        .branch = list_request_c.branch.asZ(),
    }, list_module.ffi.allocator()) catch |err| {
        if (err == error.Cancelled) list_module.ffi.global_cancel.store(true, .release);
        return @intFromEnum(fromError(err, Operation.list));
    };

    out_c.* = .{ .ptr = commit_entries.ptr, .len = commit_entries.len };
    return @intFromEnum(ErrorCode.ok);
}

pub fn get_commits_count(out_c: *CArray(CCommitEntry)) callconv(.c) usize {
    return out_c.len;
}

pub fn get_commit_at(array_c: *CArray(CPackageMeta), index: usize, out: ?*?*CPackageMeta) callconv(.c) i32 {
    const out_ptr = out orelse return @intFromEnum(fromError(error.InvalidEntry, Operation.list));
    if (index >= array_c.len) return @intFromEnum(fromError(error.InvalidEntry, Operation.list));

    out_ptr.* = &array_c.ptr[index];

    return @intFromEnum(ErrorCode.ok);
}

pub fn get_commit_slice_field(out_c: *CCommitEntry, field: u8, out: ?*CSlice) callconv(.c) i32 {
    const out_ptr = out orelse return @intFromEnum(fromError(error.InvalidEntry, Operation.list));

    const result = switch (field) {
        0 => out_c.checksum,
        1 => out_c.subject,
        else => return @intFromEnum(fromError(error.InvalidEntry, Operation.list)),
    };

    out_ptr.* = result;
    return @intFromEnum(ErrorCode.ok);
}

pub fn commits_free(out_c: *CArray(CCommitEntry)) callconv(.c) void {
    const allocator = list_module.ffi.allocator();
    const entries = out_c.toSlice();
    for (entries) |entry| {
        allocator.free(entry.checksum.toSlice());
        allocator.free(entry.subject.toSlice());
    }
    allocator.free(entries);
}
