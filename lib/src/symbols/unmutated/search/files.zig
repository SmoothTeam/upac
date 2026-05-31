const ffi = @import("upac-ffi");
const CUnmutatedRequest = ffi.CUnmutatedRequest;
const CUnmutatedResponse = ffi.CUnmutatedResponse;

const types = @import("upac-types");
const ErrorCode = types.ErrorCode;
const Operation = types.Operation;
const fromError = types.fromError;

const SearchFilesMachine = @import("upac-search-files").SearchFilesMachine;

pub fn search_files(request_c: CUnmutatedRequest, out_c: *CUnmutatedResponse) callconv(.c) i32 {
    const required = [_]ffi.CSlice{ request_c.root_path, request_c.search };
    for (required) |field| if (field.len == 0 or field.ptr[field.len] != 0) return @intFromEnum(fromError(error.InvalidEntry, Operation.search));

    const results = SearchFilesMachine.run(.{
        .root_path = request_c.root_path.asZ(),
        .query = request_c.search.toSlice(),
        .cancel_token = request_c.cancel_token orelse return @intFromEnum(fromError(error.InvalidEntry, Operation.search)),
    }, ffi.getAllocator()) catch |err| return @intFromEnum(fromError(err, Operation.search));

    out_c.files = .{ .ptr = results.ptr, .len = results.len };

    return @intFromEnum(ErrorCode.ok);
}
