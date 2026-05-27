const std = @import("std");

const c_libs = @import("c-libs");

const CCommitEntry = @import("upac-ffi").CCommitEntry;

const commit = @import("../commit.zig");
const CommitEntry = commit.CommitEntry;

const CommitMachine = commit.CommitMachine;
const CommitError = commit.CommitError;

const utils = @import("utils.zig");
const convertCommitEntry = utils.convertCommitEntry;
const dupeRow = utils.dupeRow;
// ── FetchState ────────────────────────────────────────────────────────────────
const FetchState = enum {
    open_repo,
    get_commits,
    convert_commits,
    close_repo,
    done,
};

// ── FetchMachine ──────────────────────────────────────────────────────────────
const FetchMachine = struct {
    commit: *CommitMachine,

    repo: ?*c_libs.OstreeRepo = null,

    commits: std.ArrayList(CommitEntry),
    converted_commits: std.ArrayList(CCommitEntry),

    fn stateFailed(self: *FetchMachine, err: CommitError) CommitError {
        for (self.commits.items) |row| row.deinit(self.commit.allocator);
        self.commits.deinit(self.commit.allocator);

        for (self.converted_commits.items) |*entry| entry.free(self.commit.allocator);
        self.converted_commits.deinit(self.commit.allocator);

        if (self.repo) |repo| {
            c_libs.g_object_unref(repo);
            self.repo = null;
        }

        return err;
    }
};

// ── Trampoline ────────────────────────────────────────────────────────────────
pub fn run(machine: *CommitMachine) CommitError![]CCommitEntry {
    var fetch_machine = FetchMachine{
        .commit = machine,
        .commits = std.ArrayList(CommitEntry).empty,
        .converted_commits = std.ArrayList(CCommitEntry).empty,
    };

    var commits: []CCommitEntry = &.{};
    var state = FetchState.open_repo;

    while (state != .done) {
        state = switch (state) {
            .open_repo => try stateOpenRepo(&fetch_machine),
            .get_commits => try stateGetCommits(&fetch_machine),
            .convert_commits => try stateConvertCommits(&fetch_machine),
            .close_repo => stateCloseRepo(&fetch_machine),
            .done => unreachable,
        };
    }

    commits = fetch_machine.converted_commits.toOwnedSlice(machine.allocator) catch return CommitError.AllocFailed;
    return commits;
}

// ── States ────────────────────────────────────────────────────────────────────
fn stateOpenRepo(machine: *FetchMachine) CommitError!FetchState {
    const gfile = c_libs.g_file_new_for_path(machine.commit.data.repo_path);
    defer c_libs.g_object_unref(gfile);

    const repo = c_libs.ostree_repo_new(gfile);
    if (c_libs.ostree_repo_open(repo, machine.commit.cancellable, &machine.commit.gerror) == 0) {
        c_libs.g_object_unref(repo);
        return machine.stateFailed(CommitError.RepoOpenFailed);
    }
    machine.repo = repo;

    return .get_commits;
}

fn stateGetCommits(machine: *FetchMachine) CommitError!FetchState {
    const repo = machine.repo orelse return machine.stateFailed(CommitError.RepoOpenFailed);

    var current_checksum: [*c]u8 = null;
    if (c_libs.ostree_repo_resolve_rev(repo, machine.commit.data.branch, 1, &current_checksum, &machine.commit.gerror) == 0) return .convert_commits;
    defer c_libs.g_free(current_checksum);

    var is_first = true;
    var checksum = current_checksum;

    while (checksum != null) {
        if (machine.commit.cancellable) |cancellable| {
            if (c_libs.g_cancellable_is_cancelled(cancellable) != 0) return machine.stateFailed(CommitError.Cancelled);
        }

        var commit_variant: ?*c_libs.GVariant = null;
        if (c_libs.ostree_repo_load_variant(repo, c_libs.OSTREE_OBJECT_TYPE_COMMIT, checksum, &commit_variant, &machine.commit.gerror) == 0) {
            if (!is_first) c_libs.g_free(checksum);
            break;
        }
        defer if (commit_variant) |variant| c_libs.g_variant_unref(variant);

        const subject_variant = c_libs.g_variant_get_child_value(commit_variant, 3);
        defer if (subject_variant) |variant| c_libs.g_variant_unref(variant);

        var subject_len: usize = 0;
        const subject_ptr = c_libs.g_variant_get_string(subject_variant, &subject_len);

        const row = dupeRow(checksum, subject_ptr[0..subject_len], machine.commit.allocator) catch return machine.stateFailed(CommitError.AllocFailed);
        machine.commits.append(machine.commit.allocator, row) catch {
            row.deinit(machine.commit.allocator);
            return machine.stateFailed(CommitError.AllocFailed);
        };

        const parent = c_libs.ostree_commit_get_parent(commit_variant);
        if (!is_first) c_libs.g_free(checksum);
        is_first = false;
        checksum = parent;
    }

    return .convert_commits;
}

fn stateConvertCommits(machine: *FetchMachine) CommitError!FetchState {
    for (machine.commits.items) |row| {
        const entry = convertCommitEntry(row);
        machine.converted_commits.append(machine.commit.allocator, entry) catch return machine.stateFailed(CommitError.AllocFailed);
    }

    machine.commits.deinit(machine.commit.allocator);
    machine.commits = std.ArrayList(CommitEntry).empty;

    return .close_repo;
}

fn stateCloseRepo(machine: *FetchMachine) FetchState {
    if (machine.repo) |repo| {
        c_libs.g_object_unref(repo);
        machine.repo = null;
    }

    return .done;
}
