const std = @import("std");

const commit = @import("../commit.zig");
const CommitMachine = commit.CommitMachine;
const CommitError = commit.CommitError;

// ── VerifyingState ────────────────────────────────────────────────────────────
const VerifyingState = enum {
    check_root,
    check_repo,
    done,
};

// ── VerifyingMachine ──────────────────────────────────────────────────────────
const VerifyingMachine = struct {
    commit: *CommitMachine,

    fn stateFailed(_: *VerifyingMachine, err: CommitError) CommitError {
        return err;
    }
};

// ── Trampoline ────────────────────────────────────────────────────────────────
pub fn run(machine: *CommitMachine) CommitError!void {
    var verifying = VerifyingMachine{ .commit = machine };

    var state = VerifyingState.check_root;
    while (state != .done) {
        state = switch (state) {
            .check_root => try stateCheckRoot(&verifying),
            .check_repo => try stateCheckRepo(&verifying),
            .done => unreachable,
        };
    }
}

// ── States ────────────────────────────────────────────────────────────────────
fn stateCheckRoot(machine: *VerifyingMachine) CommitError!VerifyingState {
    const root_path = std.mem.span(machine.commit.data.root_path);

    std.Io.Dir.accessAbsolute(machine.commit.io, root_path, .{}) catch return CommitError.PathNotFound;

    return .check_repo;
}

fn stateCheckRepo(machine: *VerifyingMachine) CommitError!VerifyingState {
    const repo_path = std.mem.span(machine.commit.data.repo_path);

    std.Io.Dir.accessAbsolute(machine.commit.io, repo_path, .{}) catch return CommitError.RepoOpenFailed;

    return .done;
}
