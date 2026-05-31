const std = @import("std");

const ffi = @import("upac-ffi");
const CArray = ffi.CArray;
const CPackageMeta = ffi.CPackageMeta;
const CUnmutatedRequest = ffi.CUnmutatedRequest;

const types = @import("upac-types");
const ErrorCode = types.ErrorCode;
const Operation = types.Operation;
const fromError = types.fromError;

const ListMachine = @import("upac-list-packages").ListMachine;

pub fn list_metas(list_request_c: CUnmutatedRequest, out_c: *CArray(CPackageMeta)) callconv(.c) i32 {
    const required = [_]ffi.CSlice{list_request_c.root_path};
    for (required) |field| if (field.len == 0 or field.ptr[field.len] != 0) return @intFromEnum(fromError(error.InvalidEntry, Operation.list));

    const metas = ListMachine.run(
        .{ .root_path = list_request_c.root_path.asZ() },
        ffi.getAllocator(),
    ) catch |err| return @intFromEnum(fromError(err, Operation.list));

    out_c.* = .{ .ptr = metas.ptr, .len = metas.len };

    return @intFromEnum(ErrorCode.ok);
}

pub fn metas_free(package_meta_array_c: *CArray(CPackageMeta)) callconv(.c) void {
    for (package_meta_array_c.toSlice()) |*meta| meta.free(ffi.getAllocator());
    ffi.getAllocator().free(package_meta_array_c.toSlice());
}
