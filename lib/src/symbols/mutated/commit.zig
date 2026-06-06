// ── Imports ─────────────────────────────────────────────────────────────────────
const ffi = @import("upac-ffi");
const CCommitRequest = ffi.CMutatedRequest;

const types = @import("upac-types");
const ErrorCode = types.ErrorCode;
const Operation = types.Operation;
const fromError = types.fromError;

const commit_module = @import("upac-commit");
const CommitData = commit_module.CommitData;
const CommitMachine = commit_module.CommitMachine;

pub fn commit(request: CCommitRequest) callconv(.c) i32 {
    request.validate() catch |err| return @intFromEnum(fromError(err, Operation.commit));

    request.message.validate() catch return @intFromEnum(fromError(error.InvalidEntry, Operation.commit));

    const commit_data = CommitData{
        .repo_path = request.repo_path.asZ(),
        .root_path = request.root_path.asZ(),
        .branch = request.branch.asZ(),
        .message = request.message.asZ(),
        .on_hook = request.on_hook,
        .hook_ctx = request.hook_ctx,
        .cancel_token = request.cancel_token orelse return @intFromEnum(fromError(error.InvalidEntry, Operation.commit)),
    };

    CommitMachine.run(commit_data, ffi.getAllocator()) catch |err| return @intFromEnum(fromError(err, Operation.commit));

    return @intFromEnum(ErrorCode.ok);
}
