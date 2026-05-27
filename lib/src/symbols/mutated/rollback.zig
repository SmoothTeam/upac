// ── Imports ─────────────────────────────────────────────────────────────────────
const std = @import("std");

const ffi = @import("upac-ffi");

const CRollbackRequest = ffi.CMutatedRequest;

const types = @import("upac-types");

const ErrorCode = types.ErrorCode;
const Operation = types.Operation;
const fromError = types.fromError;

const rollback_module = @import("upac-rollback");
const RollbackData = rollback_module.RollbackData;
const RollbackMachine = rollback_module.RollbackMachine;

// Reverts the system state to a specific commit hash in the OSTree repository
pub fn rollback(rollback_request_c: CRollbackRequest) callconv(.c) i32 {
    rollback_request_c.validate() catch |err| return @intFromEnum(fromError(err, Operation.rollback));

    if (rollback_request_c.commit_hash.len == 0) return @intFromEnum(fromError(error.InvalidEntry, Operation.rollback));
    rollback_request_c.commit_hash.validate() catch return @intFromEnum(fromError(error.InvalidEntry, Operation.rollback));

    const rollback_data = RollbackData{
        .root_path = rollback_request_c.root_path.asZ(),
        .repo_path = rollback_request_c.repo_path.asZ(),
        .branch = rollback_request_c.branch.asZ(),
        .commit_hash = rollback_request_c.commit_hash.asZ(),
        .cancel_token = rollback_request_c.cancel_token orelse return @intFromEnum(fromError(error.InvalidEntry, Operation.rollback)),
    };

    RollbackMachine.run(rollback_data, ffi.getAllocator()) catch |err| return @intFromEnum(fromError(err, Operation.rollback));

    return @intFromEnum(ErrorCode.ok);
}
