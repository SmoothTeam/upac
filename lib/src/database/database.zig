const std = @import("std");
const lmdbx = @import("lmdbx");

const types = @import("upac-types");
const prefix = types.paths.prefix;
const database_path = types.paths.db_path;

pub const packages = @import("packages.zig");
pub const files = @import("files.zig");

const database_config = @import("../../../config/database.zon");

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
    packages_dbi: ?lmdbx.Dbi,
    files_dbi: ?lmdbx.Dbi,

    allocator: std.mem.Allocator,

    // Opens an existing database. Fails if DBIs don't exist (init was not run).
    pub fn open(allocator: std.mem.Allocator, root_path: []const u8) DatabaseError!Database {
        const path = std.fs.path.joinZ(allocator, &.{ root_path, prefix, database_path }) catch return DatabaseError.AllocZFailed;
        defer allocator.free(path);

        const environment = lmdbx.Environment.open(path, .{ .max_dbs = 2 }) catch return DatabaseError.ReadError;

        const setup_transaction = environment.begin(.readwrite) catch return DatabaseError.WriteError;
        errdefer setup_transaction.abort();

        const packages_dbi = setup_transaction.openDatabase(database_config.packages_dbi, .{}) catch return DatabaseError.ReadError;
        const files_dbi = setup_transaction.openDatabase(database_config.files_dbi, .{}) catch return DatabaseError.ReadError;

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
        const create_transaction = self.environment.begin(.readwrite) catch return DatabaseError.WriteError;
        errdefer create_transaction.abort();
        _ = create_transaction.openDatabase(database_config.packages_dbi, .{ .create = true }) catch return DatabaseError.WriteError;
        create_transaction.commit() catch return DatabaseError.WriteError;
    }

    pub fn createFilesDbi(self: Database) DatabaseError!void {
        const create_transaction = self.environment.begin(.readwrite) catch return DatabaseError.WriteError;
        errdefer create_transaction.abort();
        _ = create_transaction.openDatabase(database_config.files_dbi, .{ .create = true }) catch return DatabaseError.WriteError;
        create_transaction.commit() catch return DatabaseError.WriteError;
    }

    pub fn close(self: Database) void {
        self.environment.close();
    }
};
