// ── Imports ─────────────────────────────────────────────────────────────────────
pub const std = @import("std");

pub const c_libs = @import("clibs");

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

const Package = types.Package;
const PackageMeta = types.PackageMeta;

const DiffKind = types.DiffKind;

const InstallStateId = types.InstallStateId;
const UninstallStateId = types.UninstallStateId;
const RollbackStateId = types.RollbackStateId;

// A C-compatible slice analogue. It stores a pointer to the data and its length. It allows for easy conversion of data between Zig and an external interface
pub const CSlice = extern struct {
    ptr: [*:0]const u8,
    len: usize,

    pub fn toSlice(self: CSlice) []const u8 {
        return self.ptr[0..self.len];
    }

    pub fn asZ(self: CSlice) [*:0]const u8 {
        return self.ptr;
    }

    pub fn fromSlice(slice: []const u8) CSlice {
        return .{ .ptr = @ptrCast(slice.ptr), .len = slice.len };
    }

    pub fn validate(self: CSlice) !void {
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

pub const CPackageEntry = extern struct {
    struct_size: usize = @sizeOf(CPackageEntry),

    meta: *anyopaque,
    temp_path: CSlice,
    checksum: CSlice,

    pub fn validate(self: CPackageEntry) !void {
        if (self.struct_size != @sizeOf(CPackageEntry)) return error.AbiMismatch;

        if (@intFromPtr(self.meta) == 0) return error.InvalidEntry;
    }
};

// A packet metadata structure adapted for transmission via C
pub const CPackageMeta = extern struct {
    struct_size: usize = @sizeOf(CPackageMeta),

    name: CSlice,
    version: CSlice,
    architecture: CSlice,
    author: CSlice,
    description: CSlice,
    license: CSlice,
    url: CSlice,
    packager: CSlice,
    checksum: CSlice,
    size: u32,
    _padding: u32 = 0,
    installed_at: i64,

    pub fn validate(self: CPackageMeta) !void {
        if (self.struct_size != @sizeOf(CPackageMeta)) return error.AbiMismatch;
    }
};

pub const CMutatedRequest = extern struct {
    struct_size: usize = @sizeOf(CMutatedRequest),

    repo_path: CSlice,
    root_path: CSlice,
    branch: CSlice,

    // Install
    packages: ?[*]const CPackageEntry = null,
    packages_count: usize = 0,

    // Uninstall
    package_names: ?[*]const CSlice = null,
    package_names_len: usize = 0,

    // Rollback
    commit_hash: CSlice,

    on_progress: ?*const fn (event: u32, ctx: ?*anyopaque) callconv(.c) void = null,
    progress_ctx: ?*anyopaque = null,

    max_retries: u8 = 0,
    cancel_token: ?*CancelToken = null,

    pub fn validate(self: CMutatedRequest) !void {
        if (self.struct_size != @sizeOf(CMutatedRequest)) return error.AbiMismatch;
        try self.repo_path.validate();
        try self.root_path.validate();
        try self.branch.validate();
    }
};

// Request structure for initializing the system with branch specification
pub const CUnmutatedRequest = extern struct {
    struct_size: usize = @sizeOf(CUnmutatedRequest),

    repo_path: CSlice,
    root_path: CSlice,
    branch: CSlice,

    from_commit_hash: CSlice,
    to_commit_hash: CSlice,

    symlinks: ?[*]const CSlice = null,
    symlinks_len: usize = 0,

    repo_mode: *anyopaque,
    cancel_token: ?*CancelToken = null,

    pub fn validate(self: CUnmutatedRequest) !void {
        if (self.struct_size != @sizeOf(CUnmutatedRequest)) return error.AbiMismatch;
        try self.repo_path.validate();
    }
};

pub const InstallProgressFn = *const fn (
    event: InstallStateId,
    ctx: ?*anyopaque,
) callconv(.c) void;

pub const CInstallProgressFn = *const fn (
    event: InstallStateId,
    ctx: ?*anyopaque,
) callconv(.c) void;

pub const UninstallProgressFn = *const fn (
    event: UninstallStateId,
    ctx: ?*anyopaque,
) callconv(.c) void;

pub const CUninstallProgressFn = *const fn (
    event: UninstallStateId,
    ctx: ?*anyopaque,
) callconv(.c) void;

pub const RollbackProgressFn = *const fn (
    event: RollbackStateId,
    ctx: ?*anyopaque,
) callconv(.c) void;

pub const CRollbackProgressFn = *const fn (
    event: RollbackStateId,
    ctx: ?*anyopaque,
) callconv(.c) void;

pub const CDiffEntry = extern struct {
    struct_size: usize = @sizeOf(CDiffEntry),

    path: CSlice,
    kind: DiffKind,
    package_name: CSlice,

    pub fn validate(self: CDiffEntry) !void {
        if (self.struct_size != @sizeOf(CDiffEntry)) return error.AbiMismatch;
        _ = intToEnum(DiffKind, @intFromEnum(self.kind)) catch return error.InvalidEntry;
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
};

// Defines the operating mode of the OSTree repository
pub const CRepoMode = enum(u8) {
    archive = 0,
    bare = 1,
    bare_user = 2,
};

// Bump this integer whenever a symbol is added/removed or a signature changes.
// struct_size guards layout; this guards the symbol set and calling conventions.
pub const ABI_VERSION: u32 = 2;

pub fn intToEnum(comptime E: type, value: anytype) error{InvalidValue}!E {
    const tag_type = @typeInfo(E).@"enum".tag_type;
    const cast_val = std.math.cast(tag_type, value) orelse return error.InvalidValue;
    inline for (std.meta.fields(E)) |field| {
        if (field.value == cast_val) return @field(E, field.name);
    }
    return error.InvalidValue;
}

pub fn allocator() std.mem.Allocator {
    return std.heap.c_allocator;
}
