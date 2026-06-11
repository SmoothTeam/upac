// ── Imports ─────────────────────────────────────────────────────────────────────
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

const update = @import("../update.zig");
const UpdateMachine = update.UpdateMachine;
const UpdateError = update.UpdateError;

const utils = @import("utils.zig");
const dirSize = utils.dirSize;

// ── VerifyingState ────────────────────────────────────────────────────────────
const VerifyingState = enum {
    check_prefix,
    check_repo,
    check_config_dirs,
    check_symlink_targets,
    check_package_temp_dirs,
    calc_size,
    open_database,
    check_installed,
    close_database,
    check_space,
    open_repo,
    check_commit,
    close_repo,
    done,
};

// ── VerifyingMachine ──────────────────────────────────────────────────────────
pub const VerifyingMachine = struct {
    updater: *UpdateMachine,

    packages_size: usize = 0,
    current_package_index: usize = 0,

    base: ?Database = null,
    repo: ?*c_libs.OstreeRepo = null,

    fn stateFailed(self: *VerifyingMachine, err: UpdateError) UpdateError {
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
pub fn run(machine: *UpdateMachine) UpdateError!void {
    var verifying_machine = VerifyingMachine{ .updater = machine };

    var state = VerifyingState.check_prefix;
    if (machine.cancellable) |cancellable| if (c_libs.g_cancellable_is_cancelled(cancellable) != 0) return verifying_machine.stateFailed(UpdateError.Cancelled);

    while (state != .done) {
        state = switch (state) {
            .check_prefix => try stateCheckPrefix(&verifying_machine),
            .check_repo => try stateCheckRepo(&verifying_machine),
            .check_config_dirs => try stateCheckConfigDirs(&verifying_machine),
            .check_symlink_targets => try stateCheckSymlinkTargets(&verifying_machine),
            .check_package_temp_dirs => try stateCheckPackageTempDirs(&verifying_machine),
            .calc_size => try stateCalcSize(&verifying_machine),
            .open_database => try stateOpenDatabase(&verifying_machine),
            .check_installed => try stateCheckInstalled(&verifying_machine),
            .close_database => stateCloseDatabase(&verifying_machine),
            .check_space => try stateCheckSpace(&verifying_machine),
            .open_repo => try stateOpenRepo(&verifying_machine),
            .check_commit => try stateCheckCommit(&verifying_machine),
            .close_repo => stateCloseRepo(&verifying_machine),
            .done => unreachable,
        };
    }
}

// ── States ────────────────────────────────────────────────────────────────────
fn stateCheckPrefix(machine: *VerifyingMachine) UpdateError!VerifyingState {
    const root_path = std.mem.span(machine.updater.data.root_path);

    const prefix_path = std.fs.path.joinZ(machine.updater.allocator, &.{ root_path, PREFIX }) catch return UpdateError.AllocZFailed;
    defer machine.updater.allocator.free(prefix_path);

    std.Io.Dir.accessAbsolute(machine.updater.io, prefix_path, .{}) catch return UpdateError.PathNotFound;

    return .check_repo;
}

fn stateCheckRepo(machine: *VerifyingMachine) UpdateError!VerifyingState {
    const repo_path = std.mem.span(machine.updater.data.repo_path);

    std.Io.Dir.accessAbsolute(machine.updater.io, repo_path, .{}) catch return UpdateError.PathNotFound;

    return .check_config_dirs;
}

fn stateCheckConfigDirs(machine: *VerifyingMachine) UpdateError!VerifyingState {
    const root_path = std.mem.span(machine.updater.data.root_path);

    const prefix_config_path = std.fs.path.join(machine.updater.allocator, &.{ root_path, PREFIX, CONFIG_DIR }) catch return UpdateError.AllocZFailed;
    defer machine.updater.allocator.free(prefix_config_path);

    const root_config_path = std.fs.path.join(machine.updater.allocator, &.{ root_path, CONFIG_DIR }) catch return UpdateError.AllocZFailed;
    defer machine.updater.allocator.free(root_config_path);

    std.Io.Dir.accessAbsolute(machine.updater.io, prefix_config_path, .{}) catch return UpdateError.PathNotFound;
    std.Io.Dir.accessAbsolute(machine.updater.io, root_config_path, .{}) catch return UpdateError.PathNotFound;

    return .check_symlink_targets;
}

fn stateCheckSymlinkTargets(machine: *VerifyingMachine) UpdateError!VerifyingState {
    const root_path = std.mem.span(machine.updater.data.root_path);

    var root_dir = std.Io.Dir.openDirAbsolute(machine.updater.io, root_path, .{ .iterate = true }) catch return UpdateError.PathNotFound;
    defer root_dir.close(machine.updater.io);

    var iter = root_dir.iterate();
    while (iter.next(machine.updater.io) catch return UpdateError.PathNotFound) |entry| {
        if (entry.kind != .sym_link) continue;

        const prefix_symlink_target = std.fs.path.join(machine.updater.allocator, &.{ root_path, PREFIX, entry.name }) catch return UpdateError.AllocZFailed;
        defer machine.updater.allocator.free(prefix_symlink_target);

        std.Io.Dir.accessAbsolute(machine.updater.io, prefix_symlink_target, .{}) catch return UpdateError.PathNotFound;
    }

    return .check_package_temp_dirs;
}

fn stateCheckPackageTempDirs(machine: *VerifyingMachine) UpdateError!VerifyingState {
    const package = machine.updater.data.packages[machine.current_package_index];
    const package_path = std.mem.span(package.temp_package_path);

    std.Io.Dir.accessAbsolute(machine.updater.io, package_path, .{}) catch return UpdateError.PathNotFound;

    return .calc_size;
}

fn stateCalcSize(machine: *VerifyingMachine) UpdateError!VerifyingState {
    const package = machine.updater.data.packages[machine.current_package_index];
    const package_path = std.mem.span(package.temp_package_path);

    machine.packages_size += dirSize(machine.updater, package_path) catch return UpdateError.CheckSpaceFailed;

    machine.current_package_index += 1;
    if (machine.current_package_index < machine.updater.data.packages.len) return .check_package_temp_dirs;

    machine.current_package_index = 0;
    return .open_database;
}

fn stateOpenDatabase(machine: *VerifyingMachine) UpdateError!VerifyingState {
    const root_path = std.mem.span(machine.updater.data.root_path);

    const db_file_path = std.fs.path.joinZ(machine.updater.allocator, &.{ root_path, PREFIX, DB_PATH, DB_NAME }) catch return UpdateError.AllocZFailed;
    defer machine.updater.allocator.free(db_file_path);

    machine.base = Database.open(machine.updater.allocator, db_file_path, false) catch |err| return machine.stateFailed(switch (err) {
        error.AccessDenied => UpdateError.AccessDenied,
        else => UpdateError.ReadDatabaseFailed,
    });

    return .check_installed;
}

fn stateCheckInstalled(machine: *VerifyingMachine) UpdateError!VerifyingState {
    const base = machine.base orelse return machine.stateFailed(UpdateError.ReadDatabaseFailed);
    const package = machine.updater.data.packages[machine.current_package_index];

    const found = packages_exists(base, package.meta.name, package.meta.arch, package.meta.arch_sub) catch return machine.stateFailed(UpdateError.ReadDatabaseFailed);
    if (found == null) return machine.stateFailed(UpdateError.PackageNotFound);

    machine.current_package_index += 1;
    if (machine.current_package_index < machine.updater.data.packages.len) return .check_installed;

    return .close_database;
}

fn stateCloseDatabase(machine: *VerifyingMachine) VerifyingState {
    if (machine.base) |*base| {
        base.close();
        machine.base = null;
    }

    return .check_space;
}

fn stateCheckSpace(machine: *VerifyingMachine) UpdateError!VerifyingState {
    var file_system_stats: c_libs.struct_statvfs = undefined;
    if (c_libs.statvfs(machine.updater.data.root_path, &file_system_stats) != 0) return machine.stateFailed(UpdateError.CheckSpaceFailed);

    const available_space: usize = @as(usize, @intCast(file_system_stats.f_bavail)) * @as(usize, @intCast(file_system_stats.f_bsize));
    if (machine.packages_size * 2 > available_space) return machine.stateFailed(UpdateError.NotEnoughSpace);

    return .open_repo;
}

fn stateOpenRepo(machine: *VerifyingMachine) UpdateError!VerifyingState {
    const gfile = c_libs.g_file_new_for_path(machine.updater.data.repo_path);
    defer c_libs.g_object_unref(gfile);

    const repo = c_libs.ostree_repo_new(gfile);
    if (c_libs.ostree_repo_open(repo, machine.updater.cancellable, &machine.updater.gerror) == 0) {
        c_libs.g_object_unref(repo);
        return machine.stateFailed(UpdateError.RepoOpenFailed);
    }
    machine.repo = repo;

    return .check_commit;
}

fn stateCheckCommit(machine: *VerifyingMachine) UpdateError!VerifyingState {
    const repo = machine.repo orelse return machine.stateFailed(UpdateError.RepoOpenFailed);

    var commit_checksum: [*c]u8 = null;
    _ = c_libs.ostree_repo_resolve_rev(repo, machine.updater.data.branch, 1, &commit_checksum, null);
    defer if (commit_checksum != null) c_libs.g_free(commit_checksum);

    if (commit_checksum == null) return machine.stateFailed(UpdateError.CommitNotFound);

    return .close_repo;
}

fn stateCloseRepo(machine: *VerifyingMachine) VerifyingState {
    if (machine.repo) |repo| {
        c_libs.g_object_unref(repo);
        machine.repo = null;
    }

    return .done;
}
