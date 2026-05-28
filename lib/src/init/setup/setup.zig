const std = @import("std");
const c_libs = @import("c-libs");

const types = @import("upac-types");
const PREFIX = types.paths.prefix;
const DB_PATH = types.paths.db_path;
const DB_NAME = types.paths.db_name;

const database = @import("upac-database");
const Database = database.Database;

const init_module = @import("../init.zig");
const InitMachine = init_module.InitMachine;
const InitError = init_module.InitError;

// ── SetupState ────────────────────────────────────────────────────────────────
const SetupState = enum {
    setup_prefix,
    setup_symlinks,
    init_ostree,
    init_database,
    done,
};

// ── SetupMachine ──────────────────────────────────────────────────────────────
const SetupMachine = struct {
    init: *InitMachine,

    current_symlink_index: usize = 0,
};

// ── Trampoline ────────────────────────────────────────────────────────────────
pub fn run(machine: *InitMachine) InitError!void {
    var setup_machine = SetupMachine{ .init = machine };

    var state = SetupState.setup_prefix;
    while (state != .done) {
        if (machine.cancellable) |cancellable| if (c_libs.g_cancellable_is_cancelled(cancellable) != 0) return InitError.Cancelled;

        state = switch (state) {
            .setup_prefix => try stateSetupPrefix(&setup_machine),
            .setup_symlinks => try stateSetupSymlinks(&setup_machine),
            .init_ostree => try stateInitOstree(&setup_machine),
            .init_database => try stateInitDatabase(&setup_machine),
            .done => unreachable,
        };
    }
}

// ── States ────────────────────────────────────────────────────────────────────
fn stateSetupPrefix(machine: *SetupMachine) InitError!SetupState {
    const root_path = std.mem.span(machine.init.data.root_path);

    const prefix_path = std.fs.path.joinZ(machine.init.allocator, &.{ root_path, PREFIX }) catch return InitError.AllocFailed;
    defer machine.init.allocator.free(prefix_path);

    std.Io.Dir.createDirAbsolute(machine.init.io, prefix_path, .default_dir) catch return InitError.CreateDirFailed;

    return .setup_symlinks;
}

fn stateSetupSymlinks(machine: *SetupMachine) InitError!SetupState {
    if (machine.current_symlink_index >= machine.init.data.symlinks.len) return .init_ostree;

    const root_path = std.mem.span(machine.init.data.root_path);
    const symlink_name = std.mem.span(machine.init.data.symlinks[machine.current_symlink_index]);

    const target_dir_path = std.fs.path.joinZ(machine.init.allocator, &.{ root_path, PREFIX, symlink_name }) catch return InitError.AllocFailed;
    defer machine.init.allocator.free(target_dir_path);

    std.Io.Dir.createDirAbsolute(machine.init.io, target_dir_path, .default_dir) catch return InitError.CreateDirFailed;

    const link_target = std.fs.path.joinZ(machine.init.allocator, &.{ PREFIX, symlink_name }) catch return InitError.AllocFailed;
    defer machine.init.allocator.free(link_target);

    const link_path = std.fs.path.joinZ(machine.init.allocator, &.{ root_path, symlink_name }) catch return InitError.AllocFailed;
    defer machine.init.allocator.free(link_path);

    std.Io.Dir.cwd().symLink(machine.init.io, link_target, link_path, .{}) catch return InitError.SymlinkFailed;

    machine.current_symlink_index += 1;
    return .setup_symlinks;
}

fn stateInitOstree(machine: *SetupMachine) InitError!SetupState {
    const repo_path = machine.init.data.repo_path;

    const repo_path_slice = std.mem.span(repo_path);
    std.Io.Dir.createDirAbsolute(machine.init.io, repo_path_slice, .default_dir) catch |err| switch (err) {
        error.PathAlreadyExists => {},
        else => return InitError.CreateDirFailed,
    };

    const gfile = c_libs.g_file_new_for_path(repo_path);
    defer c_libs.g_object_unref(gfile);

    const repo = c_libs.ostree_repo_new(gfile);
    defer c_libs.g_object_unref(repo);

    const ostree_mode: c_libs.OstreeRepoMode = switch (machine.init.data.repo_mode) {
        .archive => c_libs.OSTREE_REPO_MODE_ARCHIVE,
        .bare => c_libs.OSTREE_REPO_MODE_BARE,
        .bare_user => c_libs.OSTREE_REPO_MODE_BARE_USER,
    };

    if (c_libs.ostree_repo_create(repo, ostree_mode, machine.init.cancellable, &machine.init.gerror) == 0) return InitError.OstreeInitFailed;
    if (c_libs.ostree_repo_prepare_transaction(repo, null, machine.init.cancellable, &machine.init.gerror) == 0) return InitError.OstreeInitFailed;

    c_libs.ostree_repo_transaction_set_ref(repo, null, machine.init.data.branch, null);

    if (c_libs.ostree_repo_commit_transaction(repo, null, machine.init.cancellable, &machine.init.gerror) == 0) {
        _ = c_libs.ostree_repo_abort_transaction(repo, machine.init.cancellable, null);
        return InitError.OstreeInitFailed;
    }

    return .init_database;
}

fn stateInitDatabase(machine: *SetupMachine) InitError!SetupState {
    const root_path = std.mem.span(machine.init.data.root_path);

    const database_dir_path = std.fs.path.joinZ(machine.init.allocator, &.{ root_path, PREFIX, DB_PATH }) catch return InitError.AllocFailed;
    defer machine.init.allocator.free(database_dir_path);

    std.Io.Dir.createDirAbsolute(machine.init.io, database_dir_path, .default_dir) catch |err| switch (err) {
        error.PathAlreadyExists => {},
        else => return InitError.CreateDirFailed,
    };

    const database_path = std.fs.path.joinZ(machine.init.allocator, &.{ root_path, PREFIX, DB_PATH, DB_NAME }) catch return InitError.AllocFailed;
    defer machine.init.allocator.free(database_path);

    const base = Database.open(machine.init.allocator, database_path) catch return InitError.DatabaseInitFailed;
    defer base.close();

    base.createPackagesDbi() catch return InitError.DatabaseInitFailed;
    base.createFilesDbi() catch return InitError.DatabaseInitFailed;

    return .done;
}
