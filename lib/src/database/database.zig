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
    AccessDenied,
    WriteError,
    ReadError,
};

pub const Database = struct {
    environment: lmdbx.Environment,

    packages_dbi: ?lmdbx.Database,
    files_dbi: ?lmdbx.Database,

    transaction: ?lmdbx.Transaction,

    allocator: std.mem.Allocator,

    pub fn create(allocator: std.mem.Allocator, path: [*:0]const u8) DatabaseError!Database {
        const environment = lmdbx.Environment.init(path, .{ .max_dbs = 2 }) catch |err| return switch (err) {
            error.ACCESS, error.PERM, error.ROFS => DatabaseError.AccessDenied,
            else => DatabaseError.WriteError,
        };
        return .{
            .environment = environment,
            .packages_dbi = null,
            .files_dbi = null,
            .transaction = null,
            .allocator = allocator,
        };
    }

    pub fn open(allocator: std.mem.Allocator, path: [*:0]const u8, write: bool) DatabaseError!Database {
        const trasaction_mode: lmdbx.Transaction.Mode = if (write) .ReadWrite else .ReadOnly;

        const environment = lmdbx.Environment.init(path, .{ .max_dbs = 2, .read_only = !write }) catch |err| return switch (err) {
            error.ACCESS, error.PERM, error.ROFS => DatabaseError.AccessDenied,
            else => DatabaseError.ReadError,
        };
        errdefer environment.deinit() catch {};

        const register_txn = lmdbx.Transaction.init(environment, .{ .mode = trasaction_mode }) catch return DatabaseError.WriteError;
        _ = lmdbx.Database.open(register_txn, database_config.packages_dbi, .{}) catch {
            register_txn.abort() catch {};
            return DatabaseError.ReadError;
        };

        _ = lmdbx.Database.open(register_txn, database_config.files_dbi, .{ .dup_sort = true }) catch {
            register_txn.abort() catch {};
            return DatabaseError.ReadError;
        };

        register_txn.commit() catch return DatabaseError.WriteError;

        const active_transaction = lmdbx.Transaction.init(environment, .{ .mode = trasaction_mode }) catch return DatabaseError.WriteError;
        errdefer active_transaction.abort() catch {};

        const packages_database_dbi = lmdbx.Database.open(active_transaction, database_config.packages_dbi, .{}) catch return DatabaseError.ReadError;

        const files_database_dbi = lmdbx.Database.open(active_transaction, database_config.files_dbi, .{ .dup_sort = true }) catch return DatabaseError.ReadError;

        return .{
            .environment = environment,
            .packages_dbi = packages_database_dbi,
            .files_dbi = files_database_dbi,
            .transaction = active_transaction,
            .allocator = allocator,
        };
    }

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
        if (self.transaction) |txn| txn.commit() catch txn.abort() catch {};
        self.environment.deinit() catch {};
    }
};
