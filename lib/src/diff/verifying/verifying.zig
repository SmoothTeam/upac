const std = @import("std");

const c_libs = @import("c-libs");

const diff = @import("../diff.zig");
const DiffMachine = diff.DiffMachine;
const DiffError = diff.DiffError;

const utils = @import("utils.zig");

// ── VerifyingState ────────────────────────────────────────────────────────────
const VerifyingState = enum {
    check_repo,
    check_tmp,
    open_repo,
    check_from_ref,
    check_to_ref,
    check_space,
    close_repo,
    done,
};

// ── VerifyingMachine ──────────────────────────────────────────────────────────
pub const VerifyingMachine = struct {
    diff: *DiffMachine,

    repo: ?*c_libs.OstreeRepo = null,

    from_checksum: [*c]u8 = null,
    to_checksum: [*c]u8 = null,

    fn stateFailed(self: *VerifyingMachine, err: DiffError) DiffError {
        if (self.from_checksum != null) c_libs.g_free(self.from_checksum);
        if (self.to_checksum != null) c_libs.g_free(self.to_checksum);
        if (self.repo) |repo| {
            c_libs.g_object_unref(repo);
            self.repo = null;
        }
        return err;
    }
};

// ── Trampoline ────────────────────────────────────────────────────────────────
pub fn run(machine: *DiffMachine) DiffError!void {
    var verifying_machine = VerifyingMachine{ .diff = machine };

    var state = VerifyingState.check_repo;
    if (machine.cancellable) |cancellable| if (c_libs.g_cancellable_is_cancelled(cancellable) != 0) return verifying_machine.stateFailed(DiffError.Cancelled);

    while (state != .done) {
        state = switch (state) {
            .check_repo => try stateCheckRepo(&verifying_machine),
            .check_tmp => try stateCheckTmp(&verifying_machine),
            .open_repo => try stateOpenRepo(&verifying_machine),
            .check_from_ref => try stateCheckFromRef(&verifying_machine),
            .check_to_ref => try stateCheckToRef(&verifying_machine),
            .check_space => try stateCheckSpace(&verifying_machine),
            .close_repo => stateCloseRepo(&verifying_machine),
            .done => unreachable,
        };
    }
}

// ── States ────────────────────────────────────────────────────────────────────
fn stateCheckRepo(machine: *VerifyingMachine) DiffError!VerifyingState {
    const repo_path = std.mem.span(machine.diff.data.repo_path);

    std.Io.Dir.accessAbsolute(machine.diff.io, repo_path, .{}) catch return DiffError.PathNotFound;

    return .check_tmp;
}

fn stateCheckTmp(machine: *VerifyingMachine) DiffError!VerifyingState {
    const tmp_path = std.mem.span(machine.diff.data.tmp_path);

    std.Io.Dir.accessAbsolute(machine.diff.io, tmp_path, .{}) catch return DiffError.PathNotFound;

    return .open_repo;
}

fn stateOpenRepo(machine: *VerifyingMachine) DiffError!VerifyingState {
    const gfile = c_libs.g_file_new_for_path(machine.diff.data.repo_path);
    defer c_libs.g_object_unref(gfile);

    const repo = c_libs.ostree_repo_new(gfile);
    machine.diff.check(c_libs.ostree_repo_open(repo, machine.diff.cancellable, &machine.diff.gerror), error.RepoOpenFailed) catch |err| {
        c_libs.g_object_unref(repo);
        return machine.stateFailed(err);
    };
    machine.repo = repo;

    return .check_from_ref;
}

fn stateCheckFromRef(machine: *VerifyingMachine) DiffError!VerifyingState {
    const repo = machine.repo orelse return machine.stateFailed(DiffError.RepoOpenFailed);

    machine.diff.check(c_libs.ostree_repo_resolve_rev(repo, machine.diff.data.from_ref, 0, &machine.from_checksum, &machine.diff.gerror), error.CommitNotFound) catch |err| return machine.stateFailed(err);

    return .check_to_ref;
}

fn stateCheckToRef(machine: *VerifyingMachine) DiffError!VerifyingState {
    const repo = machine.repo orelse return machine.stateFailed(DiffError.RepoOpenFailed);

    machine.diff.check(c_libs.ostree_repo_resolve_rev(repo, machine.diff.data.to_ref, 0, &machine.to_checksum, &machine.diff.gerror), error.CommitNotFound) catch |err| return machine.stateFailed(err);

    return .check_space;
}

fn stateCheckSpace(machine: *VerifyingMachine) DiffError!VerifyingState {
    const repo = machine.repo orelse return machine.stateFailed(DiffError.RepoOpenFailed);

    const from_database_size = utils.commitDbDirSize(repo, machine.from_checksum, machine.diff.cancellable, machine.diff.allocator);

    const to_database_size = utils.commitDbDirSize(repo, machine.to_checksum, machine.diff.cancellable, machine.diff.allocator);

    var file_system_stats: c_libs.struct_statvfs = undefined;
    if (c_libs.statvfs(machine.diff.data.tmp_path, &file_system_stats) != 0) return machine.stateFailed(DiffError.CheckSpaceFailed);

    const available_space: usize = @as(usize, @intCast(file_system_stats.f_bavail)) * @as(usize, @intCast(file_system_stats.f_bsize));
    if (from_database_size + to_database_size > available_space) return machine.stateFailed(DiffError.NotEnoughSpace);

    return .close_repo;
}

fn stateCloseRepo(machine: *VerifyingMachine) VerifyingState {
    if (machine.from_checksum != null) c_libs.g_free(machine.from_checksum);
    if (machine.to_checksum != null) c_libs.g_free(machine.to_checksum);

    if (machine.repo) |repo| {
        c_libs.g_object_unref(repo);
        machine.repo = null;
    }

    return .done;
}
