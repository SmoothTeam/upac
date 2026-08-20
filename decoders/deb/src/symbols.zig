// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later

// ── Imports ───────────────────────────────────────────────────────────────────
const std = @import("std");

const types = @import("upac-backend-types");
const BackendErrorCode = types.BackendErrorCode;
const fromError = types.fromError;
const BackendError = types.BackendError;
const PrepareData = types.PrepareData;

const ffi = @import("upac-backend-ffi");
const CPrepareRequest = ffi.CPrepareRequest;
const CPackageMeta = ffi.CPackageMeta;
const CVersion = ffi.CVersion;
const CVersionParts = ffi.CVersionParts;
const CSlice = ffi.CSlice;

const dupeToCSlice = ffi.dupeToCSlice;
const dupeRequiredToCSlice = ffi.dupeRequiredToCSlice;

const BackendMachine = @import("backend/backend.zig").BackendMachine;

// ── FFI exports ───────────────────────────────────────────────────────────────
pub export fn prepare(request_c: *const CPrepareRequest, out_meta: **CPackageMeta, out_temp_path: *CSlice) callconv(.c) i32 {
    request_c.validate() catch |err| return @intFromEnum(fromError(err));

    const cancel_token = request_c.cancel_token orelse return @intFromEnum(BackendErrorCode.invalid_entry);

    const prepare_data = PrepareData{
        .package_path_c = request_c.package_path.asZ(),
        .temp_path_c = request_c.temp_dir.asZ(),
        .checksum = request_c.checksum.toSlice(),
        .on_hook = request_c.on_hook,
        .hook_ctx = request_c.hook_ctx,
        .cancel_token = cancel_token,
    };

    var result = BackendMachine.run(prepare_data, std.heap.c_allocator) catch |err| return @intFromEnum(fromError(err));
    defer result.meta.deinit(std.heap.c_allocator);

    const version_parts_copy = std.heap.c_allocator.dupe(u32, result.meta.version.parts) catch return @intFromEnum(BackendErrorCode.alloc_failed);

    const out_meta_ptr = std.heap.c_allocator.create(CPackageMeta) catch {
        std.heap.c_allocator.free(version_parts_copy);
        return @intFromEnum(BackendErrorCode.alloc_failed);
    };

    out_meta_ptr.* = CPackageMeta{
        .name = dupeRequiredToCSlice(std.heap.c_allocator, result.meta.name) catch return @intFromEnum(fromError(BackendError.InvalidPackage)),
        .version = CVersion{
            .epoch = result.meta.version.epoch,
            .release = result.meta.version.release,
            .parts = .{ .ptr = version_parts_copy.ptr, .len = version_parts_copy.len },
            .pre = CSlice.fromSlice(if (result.meta.version.pre) |pre|
                std.heap.c_allocator.dupeZ(u8, pre) catch return @intFromEnum(BackendErrorCode.alloc_failed)
            else
                null),
        },
        .arch = dupeToCSlice(std.heap.c_allocator, result.meta.arch) catch return @intFromEnum(fromError(BackendError.AllocZFailed)),
        .arch_sub = CSlice.fromSlice(null),
        .maintainer = dupeToCSlice(std.heap.c_allocator, result.meta.author) catch return @intFromEnum(fromError(BackendError.AllocZFailed)),
        .description = dupeToCSlice(std.heap.c_allocator, result.meta.description) catch return @intFromEnum(fromError(BackendError.AllocZFailed)),
        .license = dupeToCSlice(std.heap.c_allocator, result.meta.license) catch return @intFromEnum(fromError(BackendError.AllocZFailed)),
        .url = dupeToCSlice(std.heap.c_allocator, result.meta.url) catch return @intFromEnum(fromError(BackendError.AllocZFailed)),
        .sha256 = result.meta.checksum,
        .installed_size = @as(u64, result.meta.size),
    };

    out_meta.* = out_meta_ptr;
    out_temp_path.* = dupeToCSlice(std.heap.c_allocator, result.temp_path) catch return @intFromEnum(fromError(BackendError.AllocZFailed));

    return @intFromEnum(BackendErrorCode.ok);
}

pub export fn cleanup(path_c: CSlice) callconv(.c) void {
    const path = path_c.toSlice();
    const io = std.Io.Threaded.global_single_threaded.io();

    std.Io.Dir.cwd().deleteTree(io, path) catch {};

    std.heap.c_allocator.free(path);
}

pub export fn free_meta(package_meta_c: *CPackageMeta) callconv(.c) void {
    package_meta_c.free(std.heap.c_allocator);
}

pub export fn version_abi() callconv(.c) u32 {
    return ffi.ABI_VERSION;
}
