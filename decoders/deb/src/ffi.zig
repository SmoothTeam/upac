// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later

pub const std = @import("std");

const types = @import("upac-backend-types");
const BackendError = types.BackendError;
const HookFn = types.HookFn;
const CancelToken = types.CancelToken;

pub const ABI_VERSION: u32 = 2;

// ── FFI types ─────────────────────────────────────────────────────────────────
pub const CSlice = extern struct {
    ptr: [*c]const u8,
    len: usize,

    pub fn toSlice(self: CSlice) []const u8 {
        const not_null_ptr = self.ptr orelse return "";
        return not_null_ptr[0..self.len];
    }

    pub fn asZ(self: CSlice) [*c]const u8 {
        return self.ptr;
    }

    pub fn fromSlice(slice: ?[]const u8) CSlice {
        const not_null_slice = slice orelse return .{ .ptr = null, .len = 0 };
        return .{ .ptr = @ptrCast(not_null_slice.ptr), .len = not_null_slice.len };
    }

    pub fn validate(self: CSlice) !void {
        if (self.ptr == null) return error.InvalidEntry;
        if (self.ptr[self.len] != 0) return error.InvalidEntry;
        if (std.mem.len(self.ptr) != self.len) return error.InvalidEntry;
    }
};

pub const CVersionParts = extern struct {
    ptr: [*]u32,
    len: usize,

    pub fn toSlice(self: CVersionParts) []u32 {
        return self.ptr[0..self.len];
    }
};

pub const CVersion = extern struct {
    struct_size: usize = @sizeOf(CVersion),

    epoch: u32,
    release: u32,
    parts: CVersionParts,
    pre: CSlice,

    pub fn deinit(self: CVersion, allocator: std.mem.Allocator) void {
        allocator.free(self.parts.toSlice());
        if (self.pre.ptr != null) allocator.free(self.pre.toSlice());
    }
};

pub const CPackageMeta = extern struct {
    struct_size: usize = @sizeOf(CPackageMeta),

    name: CSlice,
    version: CVersion,
    arch: CSlice,
    arch_sub: CSlice,
    maintainer: CSlice,
    description: CSlice,
    license: CSlice,
    url: CSlice,
    sha256: [32]u8,
    installed_size: u64 = 0,

    pub fn free(self: *CPackageMeta, allocator: std.mem.Allocator) void {
        inline for (std.meta.fields(CPackageMeta)) |field| {
            if (field.type == CSlice) {
                const slice = @field(self, field.name);
                if (slice.ptr != null) allocator.free(slice.toSlice());
            }
        }
        self.version.deinit(allocator);
        allocator.destroy(self);
    }
};

pub const CPrepareRequest = extern struct {
    struct_size: usize = @sizeOf(CPrepareRequest),
    checksum: CSlice,

    package_path: CSlice,
    temp_dir: CSlice,

    on_hook: ?*const HookFn = null,
    hook_ctx: ?*anyopaque = null,

    cancel_token: ?*const CancelToken = null,

    pub fn validate(req: CPrepareRequest) !void {
        if (req.struct_size != @sizeOf(CPrepareRequest)) return error.AbiMismatch;
        try req.package_path.validate();
        try req.temp_dir.validate();
        try req.checksum.validate();
    }
};

pub fn dupeToCSlice(allocator: std.mem.Allocator, slice: []const u8) BackendError!CSlice {
    const duped = allocator.dupeZ(u8, slice) catch return BackendError.AllocZFailed;
    return CSlice.fromSlice(duped);
}

pub fn dupeRequiredToCSlice(allocator: std.mem.Allocator, slice: []const u8) BackendError!CSlice {
    if (slice.len == 0) return BackendError.InvalidPackage;
    return dupeToCSlice(allocator, slice);
}
