const std = @import("std");

const c_libs = @import("c-libs");

const PREFIX = @import("upac-types").paths.prefix;

const rollback = @import("../rollback.zig");

const RollbackMachine = rollback.RollbackMachine;
const RollbackError = rollback.RollbackError;

const dirSize = @import("utils.zig").dirSize;

// ── VerifyingState ────────────────────────────────────────────────────────────
const VerifyingState = enum {
    check_root,
    check_repo,
    check_prefix_dir,
    open_repo,
    load_commit,
    close_repo,
    check_space,
    done,
};

// ── VerifyingMachine ──────────────────────────────────────────────────────────
pub const VerifyingMachine = struct {
    rollback: *RollbackMachine,

    prefix_size: usize = 0,

    repo: ?*c_libs.OstreeRepo = null,

    fn stateFailed(self: *VerifyingMachine, err: RollbackError) RollbackError {
        if (self.repo) |repo| {
            c_libs.g_object_unref(repo);
            self.repo = null;
        }
        return err;
    }
};

// ── Trampoline ────────────────────────────────────────────────────────────────
pub fn run(machine: *RollbackMachine) RollbackError!void {
    var verifying_machine = VerifyingMachine{ .rollback = machine };

    var state = VerifyingState.check_root;
    while (state != .done) {
        state = switch (state) {
            .check_root => try stateCheckRoot(&verifying_machine),
            .check_repo => try stateCheckRepo(&verifying_machine),
            .check_prefix_dir => try stateCheckPrefixDir(&verifying_machine),
            .open_repo => try stateOpenRepo(&verifying_machine),
            .load_commit => try stateLoadCommit(&verifying_machine),
            .close_repo => stateCloseRepo(&verifying_machine),
            .check_space => try stateCheckSpace(&verifying_machine),
            .done => unreachable,
        };
    }
}

// ── States ────────────────────────────────────────────────────────────────────
fn stateCheckRoot(machine: *VerifyingMachine) RollbackError!VerifyingState {
    const root_path = std.mem.span(machine.rollback.data.root_path);

    std.Io.Dir.accessAbsolute(machine.rollback.io, root_path, .{}) catch return RollbackError.PathNotFound;

    return .check_repo;
}

fn stateCheckRepo(machine: *VerifyingMachine) RollbackError!VerifyingState {
    const repo_path = std.mem.span(machine.rollback.data.repo_path);

    std.Io.Dir.accessAbsolute(machine.rollback.io, repo_path, .{}) catch return RollbackError.RepoOpenFailed;

    return .check_prefix_dir;
}

fn stateCheckPrefixDir(machine: *VerifyingMachine) RollbackError!VerifyingState {
    const root_path = std.mem.span(machine.rollback.data.root_path);

    const prefix_directory = std.fs.path.join(machine.rollback.allocator, &.{ root_path, PREFIX }) catch return RollbackError.AllocZFailed;
    defer machine.rollback.allocator.free(prefix_directory);

    std.Io.Dir.accessAbsolute(machine.rollback.io, prefix_directory, .{}) catch return RollbackError.PathNotFound;

    machine.prefix_size = dirSize(machine.rollback, prefix_directory) catch return RollbackError.CheckSpaceFailed;

    return .open_repo;
}

fn stateOpenRepo(machine: *VerifyingMachine) RollbackError!VerifyingState {
    var head_checksum: [*c]u8 = null;
    defer if (head_checksum) |checksum| c_libs.g_free(checksum);

    const gfile = c_libs.g_file_new_for_path(machine.rollback.data.repo_path);
    defer c_libs.g_object_unref(gfile);

    const repo = c_libs.ostree_repo_new(gfile);
    if (c_libs.ostree_repo_open(repo, machine.rollback.cancellable, &machine.rollback.gerror) == 0) {
        c_libs.g_object_unref(repo);
        return RollbackError.RepoOpenFailed;
    }
    machine.repo = repo;

    if (c_libs.ostree_repo_resolve_rev(repo, machine.rollback.data.branch, 0, &head_checksum, &machine.rollback.gerror) == 0) return machine.stateFailed(RollbackError.CommitNotFound);

    return .load_commit;
}

fn stateLoadCommit(machine: *VerifyingMachine) RollbackError!VerifyingState {
    const repo = machine.repo orelse return machine.stateFailed(RollbackError.RepoOpenFailed);

    var has_object: c_libs.gboolean = 0;
    _ = c_libs.ostree_repo_has_object(repo, c_libs.OSTREE_OBJECT_TYPE_COMMIT, machine.rollback.data.commit_hash, &has_object, machine.rollback.cancellable, null);
    if (has_object == 0) return machine.stateFailed(RollbackError.CommitNotFound);

    return .close_repo;
}

fn stateCloseRepo(machine: *VerifyingMachine) VerifyingState {
    if (machine.repo) |repo| {
        c_libs.g_object_unref(repo);
        machine.repo = null;
    }

    return .check_space;
}

fn stateCheckSpace(machine: *VerifyingMachine) RollbackError!VerifyingState {
    var file_system_stats: c_libs.struct_statvfs = undefined;
    if (c_libs.statvfs(machine.rollback.data.root_path, &file_system_stats) != 0) return RollbackError.CheckSpaceFailed;

    const available_space: usize = @as(usize, @intCast(file_system_stats.f_bavail)) * @as(usize, @intCast(file_system_stats.f_bsize));
    if (machine.prefix_size > available_space) return RollbackError.NotEnoughSpace;

    return .done;
}
