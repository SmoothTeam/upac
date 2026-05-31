// ── Imports ─────────────────────────────────────────────────────────────────────
const std = @import("std");

const ffi = @import("upac-ffi");
const CSlice = ffi.CSlice;
const CArray = ffi.CArray;
const CDiffRequest = ffi.CUnmutatedRequest;

const CDiffEntry = ffi.CDiffEntry;

const types = @import("upac-types");
const ErrorCode = types.ErrorCode;
const Operation = types.Operation;

const fromError = types.fromError;

const DiffMachine = @import("upac-diff").DiffMachine;

pub fn diff_files(diff_request_c: CDiffRequest, out_c: *CArray(CDiffEntry)) callconv(.c) i32 {
    const required = [_]CSlice{ diff_request_c.repo_path, diff_request_c.from_commit_hash, diff_request_c.to_commit_hash };
    for (required) |required_field| if (required_field.len == 0 or required_field.ptr[required_field.len] != 0) return @intFromEnum(fromError(error.InvalidEntry, Operation.diff));

    const diff_entrys = DiffMachine.run(.{
        .repo_path = diff_request_c.repo_path.asZ(),
        .tmp_path = diff_request_c.tmp_path.asZ(),
        .from_ref = diff_request_c.from_commit_hash.asZ(),
        .to_ref = diff_request_c.to_commit_hash.asZ(),
        .cancel_token = diff_request_c.cancel_token orelse return @intFromEnum(fromError(error.InvalidEntry, Operation.diff)),
    }, ffi.getAllocator()) catch |diff_err| return @intFromEnum(fromError(diff_err, Operation.diff));

    out_c.* = .{
        .ptr = diff_entrys.ptr,
        .len = diff_entrys.len,
    };
    return @intFromEnum(ErrorCode.ok);
}

pub fn diff_files_free(out_c: *CArray(CDiffEntry)) callconv(.c) void {
    for (out_c.toSlice()) |*entry| entry.free(ffi.getAllocator());
    ffi.getAllocator().free(out_c.toSlice());
}
