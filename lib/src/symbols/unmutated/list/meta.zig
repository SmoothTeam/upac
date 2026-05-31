const std = @import("std");

const ffi = @import("upac-ffi");
const CUnmutatedRequest = ffi.CUnmutatedRequest;
const CUnmutatedResponse = ffi.CUnmutatedResponse;

const types = @import("upac-types");
const ErrorCode = types.ErrorCode;
const Operation = types.Operation;
const fromError = types.fromError;

const ListMachine = @import("upac-list-packages").ListMachine;

pub fn list_metas(request_c: CUnmutatedRequest, out_c: *CUnmutatedResponse) callconv(.c) i32 {
    const required = [_]ffi.CSlice{request_c.root_path};
    for (required) |field| if (field.len == 0 or field.ptr[field.len] != 0) return @intFromEnum(fromError(error.InvalidEntry, Operation.list));

    const metas = ListMachine.run(
        .{ .root_path = request_c.root_path.asZ() },
        ffi.getAllocator(),
    ) catch |err| return @intFromEnum(fromError(err, Operation.list));

    out_c.metas = .{ .ptr = metas.ptr, .len = metas.len };

    return @intFromEnum(ErrorCode.ok);
}
