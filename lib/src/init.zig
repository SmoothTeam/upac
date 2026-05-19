// ── Imports ───────────────────────────────────────────────────────────────────
const std = @import("std");
const c_libs = @import("c-libs");

const zqlite = @import("zqlite");

const schema = @import("upac-schema");

const ffi = @import("upac-ffi");
const CancelToken = ffi.CancelToken;
const cancelGCancellable = ffi.cancelGCancellable;
const CRepoMode = ffi.CRepoMode;

const types = @import("upac-types");

const PREFIX = types.PREFIX;
const DB_RELATIVE_PATH = types.DB_RELATIVE_PATH;
const SCHEMA_RELATIVE_PATH = types.SCHEMA_RELATIVE_PATH;

const InitStateId = types.InitStateId;

// ── Errors ────────────────────────────────────────────────────────────────────
pub const InitError = error{
    AlreadyInitialized,
    RootNotFound,
    PrefixNotFound,
    NotADirectory,
    CreateDirFailed,
    OstreeInitFailed,
    DirectoryNotEmpty,
    SymlinkFailed,
    DatabaseInitFailed,
    AllocZFailed,
    OutOfMemory,
    Cancelled,
};

// ── Data ──────────────────────────────────────────────────────────────────────
pub const InitData = struct {
    root_path: [*:0]const u8,
    repo_path: [*:0]const u8,
    repo_mode: CRepoMode,
    branch: [*:0]const u8,
    symlinks: []const []const u8,
    cancel_token: *CancelToken,
};

// ── Machine ───────────────────────────────────────────────────────────────────
pub const InitMachine = struct {
    data: InitData,

    current_symlink_index: usize = 0,

    cancellable: ?*c_libs.GCancellable = null,
    gerror: ?*c_libs.GError = null,

    allocator: std.mem.Allocator,
    io: std.Io,

    pub fn deinit(self: *InitMachine) void {
        if (self.gerror) |err| c_libs.g_error_free(err);
        if (self.cancellable) |cancellable| c_libs.g_object_unref(cancellable);
    }

    pub fn run(init_data: InitData, allocator: std.mem.Allocator) InitError!void {
        var machine = InitMachine{
            .data = init_data,

            .cancellable = c_libs.g_cancellable_new() orelse return InitError.OutOfMemory,

            .allocator = allocator,
            .io = std.Io.Threaded.global_single_threaded.io(),
        };
        defer machine.deinit();

        init_data.cancel_token.hook = cancelGCancellable;
        init_data.cancel_token.hook_ctx = machine.cancellable;
        defer init_data.cancel_token.reset();

        var state = InitStateId.check_root;
        while (state != .done) {
            if (c_libs.g_cancellable_is_cancelled(machine.cancellable.?) != 0) return InitError.Cancelled;

            state = switch (state) {
                .check_root => try stateCheckRoot(&machine),
                .setup_prefix => try stateSetupPrefix(&machine),
                .setup_symlinks => try stateSetupSymlinks(&machine),
                .check_repo => try stateCheckRepo(&machine),
                .init_ostree => try stateInitOstree(&machine),
                .init_db => try stateInitDb(&machine),
                .done, .failed => break,
            };
        }
    }
};

// ── States ────────────────────────────────────────────────────────────────────
fn stateCheckRoot(machine: *InitMachine) InitError!InitStateId {
    const root_path = std.mem.span(machine.data.root_path);
    std.Io.Dir.accessAbsolute(machine.io, root_path, .{}) catch return InitError.RootNotFound;
    return .setup_prefix;
}

fn stateSetupPrefix(machine: *InitMachine) InitError!InitStateId {
    const root_path = std.mem.span(machine.data.root_path);

    const prefix_path = std.fs.path.joinZ(machine.allocator, &.{ root_path, PREFIX }) catch return InitError.AllocZFailed;
    defer machine.allocator.free(prefix_path);

    const stat = std.Io.Dir.cwd().statFile(machine.io, prefix_path, .{}) catch |err| switch (err) {
        error.FileNotFound => {
            std.Io.Dir.createDirAbsolute(machine.io, prefix_path, .default_dir) catch return InitError.CreateDirFailed;
            return .setup_symlinks;
        },
        else => return InitError.CreateDirFailed,
    };

    if (stat.kind != .directory) return InitError.PrefixNotFound;
    return .setup_symlinks;
}

fn stateSetupSymlinks(machine: *InitMachine) InitError!InitStateId {
    if (machine.current_symlink_index >= machine.data.symlinks.len) {
        machine.current_symlink_index = 0;
        return .check_repo;
    }

    const root_path = std.mem.span(machine.data.root_path);
    const symlink_name = machine.data.symlinks[machine.current_symlink_index];

    const target_dir_path = std.fs.path.joinZ(machine.allocator, &.{ root_path, PREFIX, symlink_name }) catch return InitError.AllocZFailed;
    defer machine.allocator.free(target_dir_path);

    std.Io.Dir.accessAbsolute(machine.io, target_dir_path, .{}) catch {
        std.Io.Dir.createDirAbsolute(machine.io, target_dir_path, .default_dir) catch return InitError.CreateDirFailed;
    };

    const link_target = std.fs.path.joinZ(machine.allocator, &.{ PREFIX, symlink_name }) catch return InitError.AllocZFailed;
    defer machine.allocator.free(link_target);

    const link_path = std.fs.path.joinZ(machine.allocator, &.{ root_path, symlink_name }) catch return InitError.AllocZFailed;
    defer machine.allocator.free(link_path);

    var readlink_buf: [std.fs.max_path_bytes]u8 = undefined;
    const existing_len = std.Io.Dir.readLinkAbsolute(machine.io, link_path, &readlink_buf) catch |err| switch (err) {
        error.FileNotFound => {
            std.Io.Dir.cwd().symLink(machine.io, link_target, link_path, .{}) catch return InitError.SymlinkFailed;
            machine.current_symlink_index += 1;
            return .setup_symlinks;
        },
        else => return InitError.SymlinkFailed,
    };

    if (!std.mem.eql(u8, readlink_buf[0..existing_len], link_target)) return InitError.SymlinkFailed;

    machine.current_symlink_index += 1;
    return .setup_symlinks;
}

fn stateCheckRepo(machine: *InitMachine) InitError!InitStateId {
    const repo_path = std.mem.span(machine.data.repo_path);

    const stat = std.Io.Dir.cwd().statFile(machine.io, repo_path, .{}) catch |err| switch (err) {
        error.FileNotFound => {
            std.Io.Dir.createDirAbsolute(machine.io, repo_path, .default_dir) catch return InitError.CreateDirFailed;
            return .init_ostree;
        },
        else => return InitError.CreateDirFailed,
    };

    if (stat.kind == .file) return InitError.NotADirectory;
    if (stat.kind != .directory) return InitError.NotADirectory;

    var repo_dir = std.Io.Dir.openDirAbsolute(machine.io, repo_path, .{ .iterate = true }) catch return InitError.CreateDirFailed;
    defer repo_dir.close(machine.io);

    var repo_dir_iterator = repo_dir.iterate();
    var is_empty = true;
    while (repo_dir_iterator.next(machine.io) catch return InitError.CreateDirFailed) |entry| {
        is_empty = false;
        if (std.mem.eql(u8, entry.name, "config")) return InitError.AlreadyInitialized;
    }
    if (!is_empty) return InitError.DirectoryNotEmpty;

    return .init_ostree;
}

fn stateInitOstree(machine: *InitMachine) InitError!InitStateId {
    const ostree_gfile = c_libs.g_file_new_for_path(machine.data.repo_path);
    defer c_libs.g_object_unref(ostree_gfile);

    const ostree_repo = c_libs.ostree_repo_new(ostree_gfile);
    defer c_libs.g_object_unref(ostree_repo);

    const ostree_mode: c_libs.OstreeRepoMode = switch (machine.data.repo_mode) {
        .archive => c_libs.OSTREE_REPO_MODE_ARCHIVE,
        .bare => c_libs.OSTREE_REPO_MODE_BARE,
        .bare_user => c_libs.OSTREE_REPO_MODE_BARE_USER,
    };

    if (c_libs.ostree_repo_create(ostree_repo, ostree_mode, machine.cancellable, &machine.gerror) == 0) return InitError.OstreeInitFailed;
    if (c_libs.ostree_repo_prepare_transaction(ostree_repo, null, machine.cancellable, &machine.gerror) == 0) return InitError.OstreeInitFailed;

    c_libs.ostree_repo_transaction_set_ref(ostree_repo, null, machine.data.branch, null);

    if (c_libs.ostree_repo_commit_transaction(ostree_repo, null, machine.cancellable, &machine.gerror) == 0) {
        _ = c_libs.ostree_repo_abort_transaction(ostree_repo, machine.cancellable, null);
        return InitError.OstreeInitFailed;
    }

    return .init_db;
}

fn stateInitDb(machine: *InitMachine) InitError!InitStateId {
    const root_path = std.mem.span(machine.data.root_path);

    inline for (.{ "share", "share/upac", DB_RELATIVE_PATH }) |dir_suffix| {
        const dir_path = std.fs.path.joinZ(machine.allocator, &.{ root_path, PREFIX, dir_suffix }) catch return InitError.AllocZFailed;
        defer machine.allocator.free(dir_path);
        std.Io.Dir.accessAbsolute(machine.io, dir_path, .{}) catch {
            std.Io.Dir.createDirAbsolute(machine.io, dir_path, .default_dir) catch return InitError.CreateDirFailed;
        };
    }

    const db_file_path = std.fs.path.joinZ(machine.allocator, &.{ root_path, DB_RELATIVE_PATH, "upac.db" }) catch return InitError.AllocZFailed;
    defer machine.allocator.free(db_file_path);

    const db_conn = zqlite.open(db_file_path, zqlite.OpenFlags.Create | zqlite.OpenFlags.ReadWrite | zqlite.OpenFlags.EXResCode) catch return InitError.DatabaseInitFailed;
    defer db_conn.close();

    db_conn.execNoArgs("PRAGMA foreign_keys = ON") catch return InitError.DatabaseInitFailed;
    db_conn.execNoArgs("PRAGMA journal_mode = WAL") catch return InitError.DatabaseInitFailed;

    const schema_dir = std.fs.path.joinZ(machine.allocator, &.{ root_path, SCHEMA_RELATIVE_PATH }) catch return InitError.AllocZFailed;
    defer machine.allocator.free(schema_dir);

    var registry = schema.Registry.load(schema_dir, machine.io, machine.allocator) catch return InitError.DatabaseInitFailed;
    defer registry.deinit();

    for (registry.parsed) |p| {
        const table = p.value;

        const unique_sqls = table.uniqueSqls(machine.allocator) catch return InitError.DatabaseInitFailed;
        defer {
            for (unique_sqls) |sql| machine.allocator.free(sql);
            machine.allocator.free(unique_sqls);
        }

        const index_sqls = table.indexSqls(machine.allocator) catch return InitError.DatabaseInitFailed;
        defer {
            for (index_sqls) |sql| machine.allocator.free(sql);
            machine.allocator.free(index_sqls);
        }

        const create_sql = table.createSql(machine.allocator) catch return InitError.DatabaseInitFailed;
        defer machine.allocator.free(create_sql);
        db_conn.execNoArgs(create_sql) catch return InitError.DatabaseInitFailed;

        for (unique_sqls) |sql| db_conn.execNoArgs(sql) catch return InitError.DatabaseInitFailed;
        for (index_sqls) |sql| db_conn.execNoArgs(sql) catch return InitError.DatabaseInitFailed;

        if (table.seedSql(machine.allocator) catch return InitError.DatabaseInitFailed) |sql| {
            defer machine.allocator.free(sql);
            db_conn.execNoArgs(sql) catch return InitError.DatabaseInitFailed;
        }
    }

    return .done;
}
