// ── Global constants ─────────────────────────────────────────────────────────
// Hard-coded paths and identifiers shared across the upac core library.
// These exist so the rest of the codebase never sprinkles magic strings like
// "usr" or "usr/share/upac/db" inline.

// The single, atomically-swappable prefix directory. Everything that should be
// part of an atomic upgrade lives under <root>/<PREFIX>/. External entries like
// `/opt` are realised as symlinks pointing inside this prefix (see init).
pub const PREFIX: [:0]const u8 = "usr";

// Path of the upac package database, relative to `root_path`.
// Always read/written as join(root_path, DB_RELATIVE_PATH).
pub const DB_RELATIVE_PATH: []const u8 = PREFIX ++ "/share/upac/db";

// The mutable configuration directory. Handled with overlay semantics on
// install/uninstall: user-modified files are preserved, package-owned
// unmodified files are replaced or removed.
pub const CONFIG_DIR: [:0]const u8 = "etc";

// The mutable var directory.
pub const VAR_DIR: [:0]const u8 = "var";

pub const std = @import("std");

const errors = @import("errors.zig");
pub const Operation = errors.Operation;
pub const ErrorCode = errors.ErrorCode;
pub const fromError = errors.fromError;

// ── Package ─────────────────────────────────────────────────────────────────────
// An aggregating structure containing metadata and a list of all files belonging to the package
pub const Package = struct {
    meta: PackageMeta,
    path: [*c]const u8,
    checksum: []const u8,

    pub fn deinit(self: *Package, allocator: std.mem.Allocator) void {
        self.meta.deinit(allocator);
        allocator.free(self.path);
        allocator.free(self.checksum);
    }
};

// Stores detailed information: version, author, description, license, installation time and etc
pub const PackageMeta = struct {
    name: []const u8,
    version: []const u8,
    size: usize,
    architecture: []const u8,
    author: []const u8,
    description: []const u8,
    license: []const u8,
    url: []const u8,
    packager: []const u8,
    installed_at: i64,
    checksum: []const u8,

    // Deinitialization methods that guarantee the release of memory allocated for dynamic strings
    pub fn deinit(self: *PackageMeta, allocator: std.mem.Allocator) void {
        allocator.free(self.name);
        allocator.free(self.version);
        allocator.free(self.architecture);
        allocator.free(self.author);
        allocator.free(self.description);
        allocator.free(self.license);
        allocator.free(self.url);
        allocator.free(self.packager);
        allocator.free(self.checksum);
    }
};

pub const AttributedDiffEntry = struct {
    path: []const u8,
    kind: DiffKind,
    package_name: []const u8,
};

// Enumeration of file system change types (added, deleted, modified)
pub const DiffKind = enum(u8) {
    added = 0,
    removed = 1,
    modified = 2,
};

// Description of the specific change: the file path and exactly what happened to it
pub const DiffEntry = struct {
    path: []const u8,
    kind: DiffKind,
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
    open_repo = 0,
    list_packages = 1,
    list_commits = 2,
    done = 3,
    failed = 4,
};

pub const DiffStateId = enum(u8) {
    open_repo = 0,
    diff_packages = 1,
    diff_files = 2,
    done = 3,
    failed = 4,
};
