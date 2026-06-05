pub const std = @import("std");

const meta_fields = @import("upac-meta-fields");

const errors = @import("errors.zig");
pub const BackendErrorCode = errors.BackendErrorCode;
pub const fromError = errors.fromError;

pub const PackageMetaField = enum {
    Package,
    Version,
    @"Installed-Size",
    Architecture,
    Description,
    License,
    Homepage,
    Maintainer,
};

// Listing specific backend errors when working with archives and metadata
pub const BackendError = error{
    ChecksumMismatch,
    ExtractionFailed,
    MetadataNotFound,
    InvalidPackage,
    ReadFailed,
    ArchiveOpenFailed,
    ArchiveReadFailed,
    ArchiveExtractFailed,
    OutOfMemory,
    TempDirFailed,
    AllocZFailed,
    Cancelled,
};

// ── Inner FSM types ───────────────────────────────────────────────────────
pub const StateId = enum(u8) {
    verifying = 0,
    extracting = 1,
    reading_meta = 2,
    special_step = 3,

    done = 4,
};

// ── Version ─────────────────────────────────────────────────────────────────
pub const Version = struct {
    epoch: u32 = 0,
    parts: []const u32,
    pre: ?[]const u8 = null,
    release: u32 = 1,

    pub fn deinit(self: *const Version, allocator: std.mem.Allocator) void {
        allocator.free(self.parts);
        if (self.pre) |pre| allocator.free(pre);
    }
};

pub const PrepareResult = struct {
    meta: PackageMeta,
    temp_path: [:0]const u8,
};

pub const HookResponse = enum(u8) {
    proceed = 0,
    cancel = 1,
};

pub const HookFn = fn (event: u32, data: ?*const anyopaque, ctx: ?*anyopaque) callconv(.c) HookResponse;

pub const PrepareData = struct {
    package_path_c: [*:0]const u8,
    temp_path_c: [*:0]const u8,

    checksum: []const u8,

    on_hook: ?*const HookFn = null,
    hook_ctx: ?*anyopaque = null,

    cancel_token: *const CancelToken,
};

// ── Public types ────────────────────────────────────────────────────────────
pub const CancelToken = extern struct {
    _flag: u8,
    _hook: ?*const fn (ctx: ?*anyopaque) callconv(.c) void = null,
    _hook_ctx: ?*anyopaque = null,

    pub fn isCancelled(self: *const CancelToken) bool {
        return @atomicLoad(u8, &self._flag, .acquire) != 0;
    }
};

// Main structure containing package metadata
pub const PackageMeta = struct {
    name: []const u8,
    version: Version,
    arch: []const u8,
    arch_sub: ?[]const u8,
    maintainer: []const u8,
    description: []const u8,
    license: ?[]const u8,
    url: ?[]const u8,
    sha256: [32]u8,
    installed_size: u64,

    pub fn deinit(self: *PackageMeta, allocator: std.mem.Allocator) void {
        allocator.free(self.name);
        allocator.free(self.arch);
        allocator.free(self.maintainer);
        allocator.free(self.description);
        self.version.deinit(allocator);

        if (self.arch_sub) |sub| allocator.free(sub);
        if (self.license) |license| allocator.free(license);
        if (self.url) |url| allocator.free(url);
    }
};

pub fn buildFieldMap() std.StaticStringMap(PackageMetaField) {
    return std.StaticStringMap(PackageMetaField).initComptime(.{
        .{ meta_fields.Package,           .Package },
        .{ meta_fields.Version,           .Version },
        .{ meta_fields.@"Installed-Size", .@"Installed-Size" },
        .{ meta_fields.Architecture,      .Architecture },
        .{ meta_fields.Description,       .Description },
        .{ meta_fields.License,           .License },
        .{ meta_fields.Homepage,          .Homepage },
        .{ meta_fields.Maintainer,        .Maintainer },
    });
}
