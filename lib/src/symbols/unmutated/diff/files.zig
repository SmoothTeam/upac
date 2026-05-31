const ffi = @import("upac-ffi");
const CSlice = ffi.CSlice;
const CUnmutatedRequest = ffi.CUnmutatedRequest;
const CUnmutatedResponse = ffi.CUnmutatedResponse;

const types = @import("upac-types");
const ErrorCode = types.ErrorCode;
const Operation = types.Operation;
const fromError = types.fromError;

const DiffMachine = @import("upac-diff-files").DiffMachine;

pub fn diff_files(request_c: CUnmutatedRequest, out_c: *CUnmutatedResponse) callconv(.c) i32 {
    const required = [_]CSlice{ request_c.repo_path, request_c.from_commit_hash, request_c.to_commit_hash };
    for (required) |field| if (field.len == 0 or field.ptr[field.len] != 0) return @intFromEnum(fromError(error.InvalidEntry, Operation.diff));

    const diff_entries = DiffMachine.run(.{
        .repo_path = request_c.repo_path.asZ(),
        .tmp_path = request_c.tmp_path.asZ(),
        .from_ref = request_c.from_commit_hash.asZ(),
        .to_ref = request_c.to_commit_hash.asZ(),
        .cancel_token = request_c.cancel_token orelse return @intFromEnum(fromError(error.InvalidEntry, Operation.diff)),
    }, ffi.getAllocator()) catch |err| return @intFromEnum(fromError(err, Operation.diff));

    out_c.files = .{ .ptr = diff_entries.ptr, .len = diff_entries.len };

    return @intFromEnum(ErrorCode.ok);
}
