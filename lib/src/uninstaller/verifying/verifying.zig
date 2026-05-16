const std = @import("std");

const c_libs = @import("c-libs");

const types = @import("upac-types");
const PREFIX = types.PREFIX;
const CONFIG_DIR = types.CONFIG_DIR;

const find = @import("upac-index").find;

const uninstaller = @import("../uninstaller.zig");

const UninstallerMachine = uninstaller.UninstallerMachine;
const UninstallerError = uninstaller.UninstallerError;

const loadCommitBody = @import("utils.zig").loadCommitBody;
// ── VerifyingState ────────────────────────────────────────────────────────────
const VerifyingState = enum {
    check_prefix,
    check_repo,
    check_config_dirs,
    open_repo,
    load_commit,
    check_installed,
    close_repo,
    done,
};

// ── VerifyingMachine ──────────────────────────────────────────────────────────
pub const VerifyingMachine = struct {
    uninstaller: *UninstallerMachine,

    packages_size: usize = 0,
    current_package_index: usize = 0,

    repo: ?*c_libs.OstreeRepo = null,

    previous_commit_checksum: [*c]u8 = null,

    fn stateFailed(self: *VerifyingMachine, err: UninstallerError) UninstallerError {
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
            .check_installed => try stateCheckInstalled(&verifying_machine),
            .open_repo => try stateOpenRepo(&verifying_machine),
            .load_commit => try stateLoadCommit(&verifying_machine),
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

    if (c_libs.ostree_repo_resolve_rev(repo, machine.uninstaller.data.branch, 0, &machine.previous_commit_checksum, null) == 0) return .close_repo;

    return .load_commit;
}

fn stateLoadCommit(machine: *VerifyingMachine) UninstallerError!VerifyingState {
    var commit_variant: ?*c_libs.GVariant = null;
    defer if (commit_variant) |variant| c_libs.g_variant_unref(variant);

    const repo = machine.repo orelse return machine.stateFailed(UninstallerError.RepoOpenFailed);
    if (c_libs.ostree_repo_load_variant(repo, c_libs.OSTREE_OBJECT_TYPE_COMMIT, machine.previous_commit_checksum, &commit_variant, &machine.uninstaller.gerror) == 0) return machine.stateFailed(UninstallerError.CommitNotFound);

    return .check_installed;
}

fn stateCheckInstalled(machine: *VerifyingMachine) UninstallerError!VerifyingState {
    const package_name = machine.uninstaller.data.package_names[machine.current_package_index];

    const body_c = try loadCommitBody(machine, machine.previous_commit_checksum);
    defer machine.uninstaller.allocator.free(body_c);

    const package_found = find(body_c, package_name, machine.uninstaller.allocator) catch return machine.stateFailed(UninstallerError.AllocZFailed);

    if (package_found == null) return machine.stateFailed(UninstallerError.PackageNotFound);

    machine.current_package_index += 1;
    if (machine.current_package_index < machine.uninstaller.data.package_names.len) return .check_installed;

    return .close_repo;
}

fn stateCloseRepo(machine: *VerifyingMachine) VerifyingState {
    if (machine.repo) |repo| {
        c_libs.g_object_unref(repo);
        machine.repo = null;
    }

    return .done;
}
