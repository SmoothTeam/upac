// ── Imports ───────────────────────────────────────────────────────────────────
const std = @import("std");

const ffi = @import("upac-ffi");

const CSlice = ffi.CSlice;
const CRepoMode = ffi.CRepoMode;
const CInitRequest = ffi.CUnmutatedRequest;

const intToEnum = ffi.intToEnum;

const types = @import("upac-types");
const ErrorCode = types.ErrorCode;
const Operation = types.Operation;
const fromError = types.fromError;

const init_module = @import("upac-init");
const InitData = init_module.InitData;
const InitMachine = init_module.InitMachine;

// ── Symbol ────────────────────────────────────────────────────────────────────
pub fn init(init_request_c: CInitRequest) callconv(.c) i32 {
    init_request_c.validate() catch |err| return @intFromEnum(fromError(err, Operation.init));

    const repo_mode: *const i32 = @ptrCast(@alignCast(init_request_c.repo_mode));
    const repo_mode_unwrapped = intToEnum(CRepoMode, repo_mode.*) catch return @intFromEnum(fromError(error.OstreeInitFailed, Operation.init));

    const required = [_]CSlice{ init_request_c.repo_path, init_request_c.root_path, init_request_c.branch };

    for (required) |field| if (field.len == 0 or field.ptr[field.len] != 0) return @intFromEnum(fromError(error.InvalidEntry, Operation.init));

    const cancel_token = init_request_c.cancel_token orelse return @intFromEnum(fromError(error.InvalidEntry, Operation.init));

    const symlinks_c = if (init_request_c.symlinks) |ptr| ptr[0..init_request_c.symlinks_len] else &.{};

    const symlinks = ffi.allocator().alloc([]const u8, symlinks_c.len) catch return @intFromEnum(ErrorCode.out_of_memory);
    defer ffi.allocator().free(symlinks);

    for (symlinks_c, 0..) |symlink, index| symlinks[index] = symlink.toSlice();

    const init_data = InitData{
        .root_path = init_request_c.root_path.asZ(),
        .repo_path = init_request_c.repo_path.asZ(),
        .repo_mode = repo_mode_unwrapped,
        .branch = init_request_c.branch.asZ(),
        .symlinks = symlinks,
        .cancel_token = cancel_token,
    };

    InitMachine.run(init_data, ffi.allocator()) catch |err| return @intFromEnum(fromError(err, Operation.init));

    return @intFromEnum(ErrorCode.ok);
}
