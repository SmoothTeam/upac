const std = @import("std");
const lmdbx = @import("lmdbx");

const types = @import("upac-types");
const prefix = types.paths.prefix;
const database_path = types.paths.db_path;

pub const packages = @import("packages.zig");
pub const files = @import("files.zig");

const database_config = @import("database.zon");

// ── Errors ────────────────────────────────────────────────────────────────────
pub const DatabaseError = error{
    PackageNotFound,
    PackageAlreadyExists,
    PackageFilesExist,
    ArchitectureNotFound,
    AllocZFailed,
    WriteError,
    ReadError,
};

pub const Database = struct {
    environment: lmdbx.Environment,
    packages_dbi: ?lmdbx.Database,
    files_dbi: ?lmdbx.Database,

    allocator: std.mem.Allocator,

    // Called only from init — opens the environment without requiring DBIs to exist.
    pub fn create(allocator: std.mem.Allocator, path: [*:0]const u8) DatabaseError!Database {
        const environment = lmdbx.Environment.init(path, .{ .max_dbs = 2 }) catch return DatabaseError.WriteError;
        return .{
            .environment = environment,
            .packages_dbi = null,
            .files_dbi = null,
            .allocator = allocator,
        };
    }

    // Opens an existing database. Fails if DBIs don't exist (init was not run).
    pub fn open(allocator: std.mem.Allocator, path: [*:0]const u8) DatabaseError!Database {
        const environment = lmdbx.Environment.init(path, .{ .max_dbs = 2 }) catch return DatabaseError.ReadError;

        const setup_transaction = lmdbx.Transaction.init(environment, .{}) catch return DatabaseError.WriteError;
        errdefer setup_transaction.abort() catch {};

        const packages_dbi = lmdbx.Database.open(setup_transaction, database_config.packages_dbi, .{}) catch return DatabaseError.ReadError;
        const files_dbi = lmdbx.Database.open(setup_transaction, database_config.files_dbi, .{ .dup_sort = true }) catch return DatabaseError.ReadError;

        setup_transaction.commit() catch return DatabaseError.WriteError;

        return .{
            .environment = environment,
            .packages_dbi = packages_dbi,
            .files_dbi = files_dbi,
            .allocator = allocator,
        };
    }

    // Called only from init — creates the DBIs for the first time.
    pub fn createPackagesDbi(self: Database) DatabaseError!void {
        const create_transaction = lmdbx.Transaction.init(self.environment, .{}) catch return DatabaseError.WriteError;
        errdefer create_transaction.abort() catch {};
        _ = lmdbx.Database.open(create_transaction, database_config.packages_dbi, .{ .create = true }) catch return DatabaseError.WriteError;
        create_transaction.commit() catch return DatabaseError.WriteError;
    }

    pub fn createFilesDbi(self: Database) DatabaseError!void {
        const create_transaction = lmdbx.Transaction.init(self.environment, .{}) catch return DatabaseError.WriteError;
        errdefer create_transaction.abort() catch {};
        _ = lmdbx.Database.open(create_transaction, database_config.files_dbi, .{ .create = true, .dup_sort = true }) catch return DatabaseError.WriteError;
        create_transaction.commit() catch return DatabaseError.WriteError;
    }

    pub fn close(self: Database) void {
        self.environment.deinit() catch {};
    }
};
