const std = @import("std");

const c_libs = @import("c-libs");

const types = @import("upac-types");
const PREFIX = types.paths.prefix;
const CONFIG_DIR = types.paths.config_dir;

const DB_PATH = types.paths.db_path;
const DB_NAME = types.paths.db_name;

const database = @import("upac-database");
const Database = database.Database;
const packages_exists = database.packages.exists;

const uninstaller = @import("../uninstaller.zig");
const UninstallerMachine = uninstaller.UninstallerMachine;
const UninstallerError = uninstaller.UninstallerError;

// ── VerifyingState ────────────────────────────────────────────────────────────
const VerifyingState = enum {
    check_prefix,
    check_repo,
    check_config_dirs,
    open_database,
    check_installed,
    close_database,
    open_repo,
    close_repo,
    done,
};

// ── VerifyingMachine ──────────────────────────────────────────────────────────
pub const VerifyingMachine = struct {
    uninstaller: *UninstallerMachine,

    base: ?Database = null,
    repo: ?*c_libs.OstreeRepo = null,

    current_package_index: usize = 0,

    fn stateFailed(self: *VerifyingMachine, err: UninstallerError) UninstallerError {
        if (self.base) |*base| {
            base.close();
            self.base = null;
        }

        if (self.repo) |repo| {
            c_libs.g_object_unref(repo);
            self.repo = null;
        }

        return err;
    }
};

// ── Trampoline ────────────────────────────────────────────────────────────────
pub fn run(machine: *UninstallerMachine) UninstallerError!void {
    var verifying_machine = VerifyingMachine{ .uninstaller = machine };

    var state = VerifyingState.check_prefix;
    if (machine.cancellable) |cancellable| if (c_libs.g_cancellable_is_cancelled(cancellable) != 0) return verifying_machine.stateFailed(UninstallerError.Cancelled);

    while (state != .done) {
        state = switch (state) {
            .check_prefix => try stateCheckPrefix(&verifying_machine),
            .check_repo => try stateCheckRepo(&verifying_machine),
            .check_config_dirs => try stateCheckConfigDirs(&verifying_machine),
            .open_database => try stateOpenDatabase(&verifying_machine),
            .check_installed => try stateCheckInstalled(&verifying_machine),
            .close_database => stateCloseDatabase(&verifying_machine),
            .open_repo => try stateOpenRepo(&verifying_machine),
            .close_repo => stateCloseRepo(&verifying_machine),
            .done => unreachable,
        };
    }
}

// ── States ────────────────────────────────────────────────────────────────────
fn stateCheckPrefix(machine: *VerifyingMachine) UninstallerError!VerifyingState {
    const root_path = std.mem.span(machine.uninstaller.data.root_path);

    const prefix_path = std.fs.path.joinZ(machine.uninstaller.allocator, &.{ root_path, PREFIX }) catch return UninstallerError.AllocZFailed;
    defer machine.uninstaller.allocator.free(prefix_path);

    std.Io.Dir.accessAbsolute(machine.uninstaller.io, prefix_path, .{}) catch return UninstallerError.PathNotFound;

    return .check_repo;
}

fn stateCheckRepo(machine: *VerifyingMachine) UninstallerError!VerifyingState {
    const repo_path = std.mem.span(machine.uninstaller.data.repo_path);

    std.Io.Dir.accessAbsolute(machine.uninstaller.io, repo_path, .{}) catch return UninstallerError.PathNotFound;

    return .check_config_dirs;
}

fn stateCheckConfigDirs(machine: *VerifyingMachine) UninstallerError!VerifyingState {
    const root_path = std.mem.span(machine.uninstaller.data.root_path);

    const prefix_config_path = std.fs.path.join(machine.uninstaller.allocator, &.{ root_path, PREFIX, CONFIG_DIR }) catch return UninstallerError.AllocZFailed;
    defer machine.uninstaller.allocator.free(prefix_config_path);

    const root_config_path = std.fs.path.join(machine.uninstaller.allocator, &.{ root_path, CONFIG_DIR }) catch return UninstallerError.AllocZFailed;
    defer machine.uninstaller.allocator.free(root_config_path);

    std.Io.Dir.accessAbsolute(machine.uninstaller.io, prefix_config_path, .{}) catch return UninstallerError.PathNotFound;
    std.Io.Dir.accessAbsolute(machine.uninstaller.io, root_config_path, .{}) catch return UninstallerError.PathNotFound;

    return .open_database;
}

fn stateOpenDatabase(machine: *VerifyingMachine) UninstallerError!VerifyingState {
    const root_path = std.mem.span(machine.uninstaller.data.root_path);

    const db_file_path = std.fs.path.joinZ(machine.uninstaller.allocator, &.{ root_path, PREFIX, DB_PATH, DB_NAME }) catch return UninstallerError.AllocZFailed;
    defer machine.uninstaller.allocator.free(db_file_path);

    machine.base = Database.open(machine.uninstaller.allocator, db_file_path, false) catch |err| return machine.stateFailed(switch (err) {
        error.AccessDenied => UninstallerError.AccessDenied,
        else => UninstallerError.ReadDatabaseFailed,
    });

    return .check_installed;
}

fn stateCheckInstalled(machine: *VerifyingMachine) UninstallerError!VerifyingState {
    const base = machine.base orelse return machine.stateFailed(UninstallerError.ReadDatabaseFailed);
    const package = machine.uninstaller.data.packages[machine.current_package_index];

    const found = packages_exists(base, package.name, package.arch, package.arch_sub) catch return machine.stateFailed(UninstallerError.ReadDatabaseFailed);
    if (found == null) return machine.stateFailed(UninstallerError.PackageNotFound);

    machine.current_package_index += 1;
    if (machine.current_package_index < machine.uninstaller.data.packages.len) return .check_installed;

    return .close_database;
}

fn stateCloseDatabase(machine: *VerifyingMachine) VerifyingState {
    if (machine.base) |*base| {
        base.close();
        machine.base = null;
    }

    return .open_repo;
}

fn stateOpenRepo(machine: *VerifyingMachine) UninstallerError!VerifyingState {
    const gfile = c_libs.g_file_new_for_path(machine.uninstaller.data.repo_path);
    defer c_libs.g_object_unref(gfile);

    const repo = c_libs.ostree_repo_new(gfile);
    if (c_libs.ostree_repo_open(repo, machine.uninstaller.cancellable, &machine.uninstaller.gerror) == 0) {
        c_libs.g_object_unref(repo);
        return machine.stateFailed(UninstallerError.RepoOpenFailed);
    }
    machine.repo = repo;

    return .close_repo;
}

fn stateCloseRepo(machine: *VerifyingMachine) VerifyingState {
    if (machine.repo) |repo| {
        c_libs.g_object_unref(repo);
        machine.repo = null;
    }

    return .done;
}
