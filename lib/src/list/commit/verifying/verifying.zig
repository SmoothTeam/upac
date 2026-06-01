const std = @import("std");

const c_libs = @import("c-libs");

const ListError = @import("upac-types").ListError;

const commit = @import("../commit.zig");
const CommitMachine = commit.CommitMachine;

// ── VerifyingState ────────────────────────────────────────────────────────────
const VerifyingState = enum {
    check_root,
    check_repo,
    open_repo,
    close_repo,
    done,
};

// ── VerifyingMachine ──────────────────────────────────────────────────────────
const VerifyingMachine = struct {
    commit: *CommitMachine,
    repo: ?*c_libs.OstreeRepo = null,

    fn stateFailed(self: *VerifyingMachine, err: ListError) ListError {
        if (self.repo) |repo| {
            c_libs.g_object_unref(repo);
            self.repo = null;
        }
        return err;
    }
};

// ── Trampoline ────────────────────────────────────────────────────────────────
pub fn run(machine: *CommitMachine) ListError!void {
    var verifying_machine = VerifyingMachine{ .commit = machine };

    var state = VerifyingState.check_root;

    while (state != .done) {
        state = switch (state) {
            .check_root => try stateCheckRoot(&verifying_machine),
            .check_repo => try stateCheckRepo(&verifying_machine),
            .open_repo => try stateOpenRepo(&verifying_machine),
            .close_repo => stateCloseRepo(&verifying_machine),
            .done => unreachable,
        };
    }
}

// ── States ────────────────────────────────────────────────────────────────────
fn stateCheckRoot(machine: *VerifyingMachine) ListError!VerifyingState {
    const root_path = std.mem.span(machine.commit.data.root_path);

    std.Io.Dir.accessAbsolute(machine.commit.io, root_path, .{}) catch return ListError.PathNotFound;

    return .check_repo;
}

fn stateCheckRepo(machine: *VerifyingMachine) ListError!VerifyingState {
    const repo_path = std.mem.span(machine.commit.data.repo_path);

    std.Io.Dir.accessAbsolute(machine.commit.io, repo_path, .{}) catch return ListError.PathNotFound;

    return .open_repo;
}

fn stateOpenRepo(machine: *VerifyingMachine) ListError!VerifyingState {
    const gfile = c_libs.g_file_new_for_path(machine.commit.data.repo_path);
    defer c_libs.g_object_unref(gfile);

    const repo = c_libs.ostree_repo_new(gfile);
    if (c_libs.ostree_repo_open(repo, machine.commit.cancellable, &machine.commit.gerror) == 0) {
        c_libs.g_object_unref(repo);
        return machine.stateFailed(ListError.RepoOpenFailed);
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
