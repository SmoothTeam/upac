const std = @import("std");

const errors = @import("errors.zig");

// ── Paths ─────────────────────────────────────────────────────────────────────
pub const paths = @import("paths.zon");

pub const Operation = errors.Operation;
pub const ErrorCode = errors.ErrorCode;
pub const fromError = errors.fromError;

// ── Version ───────────────────────────────────────────────────────────────────
// Normalised by the backend before passing through FFI.
pub const Version = struct {
    epoch: u32 = 0,
    parts: []const u32,
    pre: ?[]const u8 = null,
    release: u32 = 1,
};

// ── Package ───────────────────────────────────────────────────────────────────
pub const Package = struct {
    meta: PackageMeta,
    temp_package_path: [*:0]const u8,

    pub fn deinit(self: *Package, allocator: std.mem.Allocator) void {
        self.meta.deinit(allocator);
        allocator.free(self.temp_package_path);
    }
};

// arch and arch_sub come from arch_map.zon — no enum, no hardcoded list.
// Backend splits the package arch string into base + variant before FFI.
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

    pub fn deinit(self: *PackageMeta, allocator: std.mem.Allocator) void {
        allocator.free(self.name);
        allocator.free(self.version.parts);
        if (self.version.pre) |pre| allocator.free(pre);
        allocator.free(self.arch);
        if (self.arch_sub) |sub| allocator.free(sub);
        allocator.free(self.maintainer);
        allocator.free(self.description);
        if (self.license) |license| allocator.free(license);
        if (self.url) |url| allocator.free(url);
    }
};

pub const FileEntry = struct {
    path: []const u8,
    sha256: [32]u8,
    is_user: bool,

    pub fn deinit(self: *FileEntry, allocator: std.mem.Allocator) void {
        allocator.free(self.path);
    }
};

pub const FileRecord = struct {
    sha256: [32]u8,
    is_user: bool,
    pkg_name: []const u8,
};

pub const DiffEntry = struct {
    path: []const u8,
    kind: DiffKind,
    package_name: []const u8,
    is_user: bool,
};

// Enumeration of file system change types (added, deleted, modified)
pub const DiffKind = enum(u8) {
    added = 0,
    removed = 1,
    modified = 2,
};

pub const InstallStateId = enum(u8) {
    verifying = 0,
    preparation = 1,
    transaction = 2,
    merge = 3,
    checkout = 4,
    swap = 5,

    done = 6,
    failed = 7,
};

pub const UninstallStateId = enum(u8) {
    verifying = 0,
    transaction = 1,
    merge = 2,
    checkout = 3,
    swap = 4,

    done = 5,
    failed = 6,
};

pub const RollbackStateId = enum(u8) {
    verifying = 0,
    merge = 1,
    checkout = 2,
    swap = 3,

    done = 4,
    failed = 5,
};

pub const ListStateId = enum(u8) {
    verifying = 0,
    list_packages = 1,
    list_commits = 2,
    done = 3,
    failed = 4,
};

pub const DiffStateId = enum(u8) {
    verifying = 0,
    preparing = 1,
    comparing = 2,

    done = 3,
    failed = 4,
};

pub const InitStateId = enum(u8) {
    check_root = 0,
    setup_prefix = 1,
    setup_symlinks = 2,
    check_repo = 3,
    init_ostree = 4,
    init_db = 5,
    done = 6,
    failed = 7,
};
