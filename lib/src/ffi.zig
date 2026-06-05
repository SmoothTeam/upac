// ── Imports ─────────────────────────────────────────────────────────────────────
const std = @import("std");

const c_libs = @import("c-libs");

const Version = @import("upac-types").Version;

pub const CancelToken = extern struct {
    _flag: u8 = 0,
    hook: ?*const fn (ctx: ?*anyopaque) callconv(.c) void = null,
    hook_ctx: ?*anyopaque = null,

    pub fn cancel(self: *CancelToken) void {
        @atomicStore(u8, &self._flag, 1, .release);
        if (self.hook) |function| function(self.hook_ctx);
    }

    pub fn isCancelled(self: *const CancelToken) bool {
        return @atomicLoad(u8, &self._flag, .acquire) != 0;
    }

    pub fn reset(self: *CancelToken) void {
        @atomicStore(u8, &self._flag, 0, .release);
        self.hook = null;
        self.hook_ctx = null;
    }
};

// Hook function passed to CancelToken to cancel an associated GCancellable.
pub fn cancelGCancellable(ctx: ?*anyopaque) callconv(.c) void {
    if (ctx) |ptr| c_libs.g_cancellable_cancel(@ptrCast(@alignCast(ptr)));
}

// ── Reimports types ─────────────────────────────────────────────────────────────────────
const types = @import("upac-types");

const DiffKind = types.DiffKind;
pub const HookResponse = types.HookResponse;

// C-compatible slice. ptr == null means absent (optional field).
pub const CSlice = extern struct {
    ptr: [*c]const u8,
    len: usize,

    // For required fields — caller guarantees ptr != null.
    pub fn toSlice(self: CSlice) []const u8 {
        const not_null_prt = self.ptr orelse return "";
        return not_null_prt[0..self.len];
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

pub fn CArray(comptime T: type) type {
    return extern struct {
        ptr: [*]T,
        len: usize,

        pub fn toSlice(self: @This()) []T {
            return self.ptr[0..self.len];
        }
    };
}

pub const CVersion = extern struct {
    epoch: u32,
    release: u32,
    parts: CArray(u32),
    pre: CSlice,

    pub fn toVersion(self: CVersion) Version {
        return .{
            .epoch = self.epoch,
            .release = self.release,
            .parts = self.parts.toSlice(),
            .pre = if (self.pre.ptr != null) self.pre.toSlice() else null,
        };
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

    pub fn validate(self: CPackageMeta) !void {
        if (self.struct_size != @sizeOf(CPackageMeta)) return error.AbiMismatch;
        try self.name.validate();
        try self.arch.validate();
        try self.arch_sub.validate();
        try self.maintainer.validate();
        try self.description.validate();
        try self.license.validate();
        try self.url.validate();
        if (self.version.parts.len == 0) return error.InvalidEntry;
    }

    pub fn free(self: *CPackageMeta, allocator: std.mem.Allocator) void {
        inline for (std.meta.fields(CPackageMeta)) |field| {
            if (field.type == CSlice) {
                const slice = @field(self, field.name);
                if (slice.ptr != null) allocator.free(slice.toSlice());
            }
        }

        if (self.version.parts.len > 0) allocator.free(self.version.parts.toSlice());
        if (self.version.pre.ptr != null) allocator.free(self.version.pre.toSlice());
    }
};

pub const CPackage = extern struct {
    struct_size: usize = @sizeOf(CPackage),

    meta: *CPackageMeta,
    temp_path: CSlice,

    pub fn validate(self: CPackage) !void {
        if (self.struct_size != @sizeOf(CPackage)) return error.AbiMismatch;
        try self.meta.validate();
        try self.temp_path.validate();
    }
};

pub const CPackageInfo = extern struct {
    struct_size: usize = @sizeOf(CPackageInfo),

    name: CSlice,
    arch: CSlice,
    arch_sub: CSlice,

    pub fn validate(self: CPackageInfo) !void {
        if (self.struct_size != @sizeOf(CPackageInfo)) return error.AbiMismatch;
        try self.name.validate();
        try self.arch.validate();
    }
};

pub const CMutatedRequest = extern struct {
    struct_size: usize = @sizeOf(CMutatedRequest),

    repo_path: CSlice,
    root_path: CSlice,
    tmp_path: CSlice,
    arch_config_path: CSlice,
    branch: CSlice,

    // Install
    packages: ?[*]const CPackage = null,
    packages_count: usize = 0,

    // Uninstall
    uninstall_packages: ?[*]const CPackageInfo = null,
    uninstall_packages_len: usize = 0,

    // Rollback
    commit_hash: CSlice,

    // Files
    files: ?[*]const CSlice = null,
    files_len: usize = 0,
    file_kind: DiffKind,
    file_package: ?*const CPackageInfo = null,

    on_hook: ?*const HookFn = null,
    hook_ctx: ?*anyopaque = null,

    max_retries: u8 = 0,
    cancel_token: ?*CancelToken = null,

    pub fn validate(self: CMutatedRequest) !void {
        if (self.struct_size != @sizeOf(CMutatedRequest)) return error.AbiMismatch;
        try self.repo_path.validate();
        try self.root_path.validate();
        try self.tmp_path.validate();
        try self.arch_config_path.validate();
        try self.branch.validate();
    }
};

// Request structure for initializing the system with branch specification
pub const CUnmutatedRequest = extern struct {
    struct_size: usize = @sizeOf(CUnmutatedRequest),

    repo_path: CSlice,
    root_path: CSlice,
    tmp_path: CSlice,
    arch_config_path: CSlice,
    branch: CSlice,

    from_commit_hash: CSlice,
    to_commit_hash: CSlice,

    search: CSlice,

    symlinks: ?[*]const CSlice = null,
    symlinks_len: usize = 0,

    repo_mode: *anyopaque,
    cancel_token: ?*CancelToken = null,

    pub fn validate(self: CUnmutatedRequest) !void {
        if (self.struct_size != @sizeOf(CUnmutatedRequest)) return error.AbiMismatch;
        try self.repo_path.validate();
        try self.root_path.validate();
        try self.tmp_path.validate();
        try self.arch_config_path.validate();
        try self.branch.validate();
    }
};

pub const HookFn = fn (event: u32, data: ?*const anyopaque, ctx: ?*anyopaque) callconv(.c) HookResponse;

pub const CHookPreInstall = extern struct {
    packages_count: u32,
    required_space: u64,
    free_space: u64,
};

pub const CDiffPackageEntry = extern struct {
    struct_size: usize = @sizeOf(CDiffPackageEntry),

    name: CSlice,
    kind: DiffKind,
    version: CVersion,

    pub fn free(self: *CDiffPackageEntry, allocator: std.mem.Allocator) void {
        if (self.name.ptr != null) allocator.free(self.name.toSlice());
        if (self.version.parts.len > 0) allocator.free(self.version.parts.toSlice());
        if (self.version.pre.ptr != null) allocator.free(self.version.pre.toSlice());
    }
};

pub const CDiffFileEntry = extern struct {
    struct_size: usize = @sizeOf(CDiffFileEntry),

    path: CSlice,
    kind: DiffKind,
    package_name: CSlice,
    is_user: bool,

    pub fn validate(self: CDiffFileEntry) !void {
        if (self.struct_size != @sizeOf(CDiffFileEntry)) return error.AbiMismatch;
        _ = intToEnum(DiffKind, @intFromEnum(self.kind)) catch return error.InvalidEntry;
    }

    pub fn free(self: *CDiffFileEntry, allocator: std.mem.Allocator) void {
        inline for (std.meta.fields(CDiffFileEntry)) |field| {
            if (field.type == CSlice) {
                const slice = @field(self, field.name);
                if (slice.ptr != null) allocator.free(slice.toSlice());
            }
        }
    }
};

//
pub const CCommitEntry = extern struct {
    struct_size: usize = @sizeOf(CCommitEntry),

    checksum: CSlice,
    subject: CSlice,

    pub fn validate(self: CCommitEntry) !void {
        if (self.struct_size != @sizeOf(CCommitEntry)) return error.AbiMismatch;
    }

    pub fn free(self: *CCommitEntry, allocator: std.mem.Allocator) void {
        inline for (std.meta.fields(CCommitEntry)) |field| {
            if (field.type == CSlice) {
                const slice = @field(self, field.name);
                if (slice.ptr != null) allocator.free(slice.toSlice());
            }
        }
    }
};

pub const CUnmutatedResponse = extern struct {
    struct_size: usize = @sizeOf(CUnmutatedResponse),

    metas: CArray(CPackageMeta),
    files: CArray(CDiffFileEntry),
    commits: CArray(CCommitEntry),
    diff_packages: CArray(CDiffPackageEntry),

    pub fn free(self: *CUnmutatedResponse, allocator: std.mem.Allocator) void {
        if (self.metas.len > 0) {
            for (self.metas.toSlice()) |*entry| entry.free(allocator);
            allocator.free(self.metas.toSlice());
        }

        if (self.files.len > 0) {
            for (self.files.toSlice()) |*entry| entry.free(allocator);
            allocator.free(self.files.toSlice());
        }

        if (self.commits.len > 0) {
            for (self.commits.toSlice()) |*entry| entry.free(allocator);
            allocator.free(self.commits.toSlice());
        }

        if (self.diff_packages.len > 0) {
            for (self.diff_packages.toSlice()) |*entry| entry.free(allocator);
            allocator.free(self.diff_packages.toSlice());
        }
    }
};

// Defines the operating mode of the OSTree repository
pub const CRepoMode = enum(u8) {
    archive = 0,
    bare = 1,
    bare_user = 2,
};

// Bump this integer whenever a symbol is added/removed or a signature changes.
// struct_size guards layout; this guards the symbol set and calling conventions.
pub const ABI_VERSION: u32 = 3;

pub fn intToEnum(comptime E: type, value: anytype) error{InvalidValue}!E {
    const tag_type = @typeInfo(E).@"enum".tag_type;
    const cast_val = std.math.cast(tag_type, value) orelse return error.InvalidValue;
    inline for (std.meta.fields(E)) |field| {
        if (field.value == cast_val) return @field(E, field.name);
    }
    return error.InvalidValue;
}

pub fn getAllocator() std.mem.Allocator {
    return std.heap.c_allocator;
}
