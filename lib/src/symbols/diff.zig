// ── Imports ─────────────────────────────────────────────────────────────────────
const std = @import("std");

const ffi = @import("upac-ffi");
const CSlice = ffi.CSlice;
const CArray = ffi.CArray;
const CDiffRequest = ffi.CUnmutatedRequest;

const CDiffEntry = ffi.CDiffEntry;

const diff_module = @import("upac-diff");

const types = @import("upac-types");
const ErrorCode = types.ErrorCode;
const Operation = types.Operation;

const fromError = types.fromError;

pub fn diff(diff_request_c: CDiffRequest, out_c: *CArray(CDiffEntry)) callconv(.c) i32 {
    const required = [_]CSlice{ diff_request_c.repo_path, diff_request_c.from_commit_hash, diff_request_c.to_commit_hash };
    for (required) |required_field| if (required_field.len == 0 or required_field.ptr[required_field.len] != 0) return @intFromEnum(fromError(error.InvalidEntry, Operation.diff));

    const attributed_diff_entrys = diff_module.DiffMachine.run(.{
        .repo_path = diff_request_c.repo_path.asZ(),
        .tmp_path = diff_request_c.tmp_path.asZ(),
        .from_ref = diff_request_c.from_commit_hash.asZ(),
        .to_ref = diff_request_c.to_commit_hash.asZ(),
        .cancel_token = diff_request_c.cancel_token orelse return @intFromEnum(fromError(error.InvalidEntry, Operation.diff)),
    }, ffi.allocator()) catch |diff_err| return @intFromEnum(fromError(diff_err, Operation.diff));

    out_c.* = .{
        .ptr = attributed_diff_entrys.ptr,
        .len = attributed_diff_entrys.len,
    };
    return @intFromEnum(ErrorCode.ok);
}

pub fn diff_free(out_c: *CArray(CDiffEntry)) callconv(.c) void {
    const attributed_diff_entrys = out_c.toSlice();
    for (attributed_diff_entrys) |attributed_diff_entry| {
        ffi.allocator().free(attributed_diff_entry.path.ptr[0 .. attributed_diff_entry.path.len + 1]);
        ffi.allocator().free(attributed_diff_entry.package_name.ptr[0 .. attributed_diff_entry.package_name.len + 1]);
    }
    ffi.allocator().free(attributed_diff_entrys);
}
