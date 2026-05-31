const ffi = @import("upac-ffi");
const CUnmutatedRequest = ffi.CUnmutatedRequest;
const CUnmutatedResponse = ffi.CUnmutatedResponse;

const types = @import("upac-types");
const ErrorCode = types.ErrorCode;
const Operation = types.Operation;
const fromError = types.fromError;

const CommitMachine = @import("upac-list-commits").CommitMachine;

pub fn list_commits(request_c: CUnmutatedRequest, out_c: *CUnmutatedResponse) callconv(.c) i32 {
    const required = [_]ffi.CSlice{ request_c.repo_path, request_c.branch };
    for (required) |field| if (field.len == 0 or field.ptr[field.len] != 0) return @intFromEnum(fromError(error.InvalidEntry, Operation.list));

    const commits = CommitMachine.run(
        .{
            .repo_path = request_c.repo_path.asZ(),
            .root_path = request_c.root_path.asZ(),
            .branch = request_c.branch.asZ(),
            .cancel_token = request_c.cancel_token orelse return @intFromEnum(fromError(error.InvalidEntry, Operation.list)),
        },
        ffi.getAllocator(),
    ) catch |err| return @intFromEnum(fromError(err, Operation.list));

    out_c.commits = .{ .ptr = commits.ptr, .len = commits.len };

    return @intFromEnum(ErrorCode.ok);
}
