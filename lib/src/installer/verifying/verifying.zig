const std = @import("std");

const installer = @import("../installer.zig");
const c_libs = installer.ffi.c_libs;

const find = installer.index.find;

const PREFIX = installer.types.PREFIX;
const CONFIG_DIR = installer.types.CONFIG_DIR;

const InstallerMachine = installer.InstallerMachine;
const InstallerError = installer.InstallerError;

const utils = @import("utils.zig");
const dirSize = utils.dirSize;
const loadCommitBody = utils.loadCommitBody;

// ── VerifyingState ────────────────────────────────────────────────────────────
const VerifyingState = enum {
    check_prefix,
    check_repo,
    check_config_dirs,
    check_package_temp_dirs,
    calc_size,
    check_space,
    open_repo,
    load_commit,
    check_installed,
    close_repo,
    done,
};

// ── VerifyingMachine ──────────────────────────────────────────────────────────
pub const VerifyingMachine = struct {
    installer: *InstallerMachine,

    packages_size: usize = 0,
    current_package_index: usize = 0,

    repo: ?*c_libs.OstreeRepo = null,

    previous_commit_checksum: [*c]u8 = null,

    fn stateFailed(self: *VerifyingMachine, err: InstallerError) InstallerError {
        if (self.repo) |repo| {
            c_libs.g_object_unref(repo);
            self.repo = null;
        }

        return err;
    }
};

// ── Trampoline ────────────────────────────────────────────────────────────────
pub fn run(machine: *InstallerMachine) InstallerError!void {
    var verifying_machine = VerifyingMachine{ .installer = machine };

    var state = VerifyingState.check_prefix;
    if (machine.cancellable) |cancellable| if (c_libs.g_cancellable_is_cancelled(cancellable) != 0) return verifying_machine.stateFailed(InstallerError.Cancelled);

    while (state != .done) {
        state = switch (state) {
            .check_prefix => try stateCheckPrefix(&verifying_machine),
            .check_repo => try stateCheckRepo(&verifying_machine),
            .check_config_dirs => try stateCheckConfigDirs(&verifying_machine),
            .check_package_temp_dirs => try stateCheckPackageTempDirs(&verifying_machine),
            .check_installed => try stateCheckInstalled(&verifying_machine),
            .calc_size => try stateCalcSize(&verifying_machine),
            .check_space => try stateCheckSpace(&verifying_machine),
            .open_repo => try stateOpenRepo(&verifying_machine),
            .load_commit => try stateLoadCommit(&verifying_machine),
            .close_repo => stateCloseRepo(&verifying_machine),
            .done => unreachable,
        };
    }
}

// ── States ────────────────────────────────────────────────────────────────────
fn stateCheckPrefix(machine: *VerifyingMachine) InstallerError!VerifyingState {
    const root_path = std.mem.span(machine.installer.data.root_path);

    const prefix_path = std.fs.path.joinZ(machine.installer.allocator, &.{ root_path, PREFIX }) catch return InstallerError.AllocZFailed;
    defer machine.installer.allocator.free(prefix_path);

    std.Io.Dir.accessAbsolute(machine.installer.io, prefix_path, .{}) catch return InstallerError.PathNotFound;

    return .check_repo;
}

fn stateCheckRepo(machine: *VerifyingMachine) InstallerError!VerifyingState {
    const repo_path = std.mem.span(machine.installer.data.repo_path);

    std.Io.Dir.accessAbsolute(machine.installer.io, repo_path, .{}) catch return InstallerError.PathNotFound;

    return .check_config_dirs;
}

fn stateCheckConfigDirs(machine: *VerifyingMachine) InstallerError!VerifyingState {
    const root_path = std.mem.span(machine.installer.data.root_path);

    const prefix_config_path = std.fs.path.join(machine.installer.allocator, &.{ root_path, PREFIX, CONFIG_DIR }) catch return InstallerError.AllocZFailed;
    defer machine.installer.allocator.free(prefix_config_path);

    const root_config_path = std.fs.path.join(machine.installer.allocator, &.{ root_path, CONFIG_DIR }) catch return InstallerError.AllocZFailed;
    defer machine.installer.allocator.free(root_config_path);

    std.Io.Dir.accessAbsolute(machine.installer.io, prefix_config_path, .{}) catch return InstallerError.PathNotFound;

    std.Io.Dir.accessAbsolute(machine.installer.io, root_config_path, .{}) catch return InstallerError.PathNotFound;

    return .check_package_temp_dirs;
}

fn stateCheckPackageTempDirs(machine: *VerifyingMachine) InstallerError!VerifyingState {
    const package = machine.installer.data.packages[machine.current_package_index];
    const package_path = std.mem.span(package.path);

    std.Io.Dir.accessAbsolute(machine.installer.io, package_path, .{}) catch return InstallerError.PathNotFound;

    return .calc_size;
}

fn stateCalcSize(machine: *VerifyingMachine) InstallerError!VerifyingState {
    const package = machine.installer.data.packages[machine.current_package_index];
    const package_path = std.mem.span(package.path);

    machine.packages_size += dirSize(machine.installer, package_path) catch return InstallerError.CheckSpaceFailed;

    machine.current_package_index += 1;
    if (machine.current_package_index < machine.installer.data.packages.len) {
        return .check_package_temp_dirs;
    }

    machine.current_package_index = 0;
    return .open_repo;
}

fn stateOpenRepo(machine: *VerifyingMachine) InstallerError!VerifyingState {
    const gfile = c_libs.g_file_new_for_path(machine.installer.data.repo_path);
    defer c_libs.g_object_unref(gfile);

    const repo = c_libs.ostree_repo_new(gfile);
    if (c_libs.ostree_repo_open(repo, machine.installer.cancellable, &machine.installer.gerror) == 0) {
        c_libs.g_object_unref(repo);
        return machine.stateFailed(InstallerError.RepoOpenFailed);
    }
    machine.repo = repo;

    if (c_libs.ostree_repo_resolve_rev(repo, machine.installer.data.branch, 0, &machine.previous_commit_checksum, null) == 0) return .close_repo;

    return .load_commit;
}

fn stateLoadCommit(machine: *VerifyingMachine) InstallerError!VerifyingState {
    var commit_variant: ?*c_libs.GVariant = null;
    defer if (commit_variant) |variant| c_libs.g_variant_unref(variant);

    const repo = machine.repo orelse return machine.stateFailed(InstallerError.RepoOpenFailed);
    if (c_libs.ostree_repo_load_variant(repo, c_libs.OSTREE_OBJECT_TYPE_COMMIT, machine.previous_commit_checksum, &commit_variant, &machine.installer.gerror) == 0) return machine.stateFailed(InstallerError.CommitNotFound);

    return .check_installed;
}

fn stateCheckInstalled(machine: *VerifyingMachine) InstallerError!VerifyingState {
    const package = machine.installer.data.packages[machine.current_package_index];

    const body_c = try loadCommitBody(machine, machine.previous_commit_checksum);
    defer machine.installer.allocator.free(body_c);

    const package_name = find(body_c, package.meta.name, machine.installer.allocator) catch return machine.stateFailed(InstallerError.AllocZFailed);

    if (package_name != null) return machine.stateFailed(InstallerError.AlreadyInstalled);

    machine.current_package_index += 1;
    if (machine.current_package_index < machine.installer.data.packages.len) {
        return .check_installed;
    }

    return .check_space;
}

fn stateCheckSpace(machine: *VerifyingMachine) InstallerError!VerifyingState {
    var file_system_stats: c_libs.struct_statvfs = undefined;
    if (c_libs.statvfs(machine.installer.data.root_path, &file_system_stats) != 0) return machine.stateFailed(InstallerError.CheckSpaceFailed);

    const available_space: usize = @as(usize, @intCast(file_system_stats.f_bavail)) * @as(usize, @intCast(file_system_stats.f_bsize));
    if (machine.packages_size * 2 > available_space) return machine.stateFailed(InstallerError.NotEnoughSpace);

    return .open_repo;
}

fn stateCloseRepo(machine: *VerifyingMachine) VerifyingState {
    if (machine.repo) |repo| {
        c_libs.g_object_unref(repo);
        machine.repo = null;
    }

    return .done;
}
