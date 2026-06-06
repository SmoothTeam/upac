const std = @import("std");

const c_libs = @import("c-libs");

const commit = @import("../commit.zig");
const CommitMachine = commit.CommitMachine;
const CommitError = commit.CommitError;

// ── TransactionState ──────────────────────────────────────────────────────────
const TransactionState = enum {
    open_repo,
    get_prev_commit,
    open_transaction,
    scan_root,
    write_mtree,
    write_commit,
    set_ref,
    close_transaction,
    close_repo,
    done,
};

// ── TransactionMachine ────────────────────────────────────────────────────────
const TransactionMachine = struct {
    commit: *CommitMachine,

    repo: ?*c_libs.OstreeRepo = null,
    mtree: ?*c_libs.OstreeMutableTree = null,
    mtree_root: ?*c_libs.GFile = null,

    previous_commit_checksum: [*c]u8 = null,
    commit_checksum: [*c]u8 = null,

    fn stateFailed(self: *TransactionMachine, err: CommitError) CommitError {
        var abort_error: ?*c_libs.GError = null;
        defer if (abort_error) |abort_err| c_libs.g_error_free(abort_err);

        if (self.repo) |repo| {
            _ = c_libs.ostree_repo_abort_transaction(repo, null, &abort_error);

            if (self.commit_checksum != null) _ = c_libs.ostree_repo_set_ref_immediate(repo, null, self.commit.data.branch, self.previous_commit_checksum, null, null);

            c_libs.g_object_unref(repo);
            self.repo = null;
        }

        if (self.mtree_root) |root| {
            c_libs.g_object_unref(root);
            self.mtree_root = null;
        }

        if (self.mtree) |mtree| {
            c_libs.g_object_unref(mtree);
            self.mtree = null;
        }

        return err;
    }
};

// ── Trampoline ────────────────────────────────────────────────────────────────
pub fn run(machine: *CommitMachine) CommitError!void {
    var transaction = TransactionMachine{
        .commit = machine,
        .mtree = c_libs.ostree_mutable_tree_new(),
    };

    var state = TransactionState.open_repo;
    while (state != .done) {
        if (machine.cancellable) |cancellable| if (c_libs.g_cancellable_is_cancelled(cancellable) != 0) return transaction.stateFailed(CommitError.Cancelled);

        state = switch (state) {
            .open_repo => try stateOpenRepo(&transaction),
            .get_prev_commit => try stateGetPrevCommit(&transaction),
            .open_transaction => try stateOpenTransaction(&transaction),
            .scan_root => try stateScanRoot(&transaction),
            .write_mtree => try stateWriteMtree(&transaction),
            .write_commit => try stateWriteCommit(&transaction),
            .set_ref => try stateSetRef(&transaction),
            .close_transaction => try stateCloseTransaction(&transaction),
            .close_repo => stateCloseRepo(&transaction),
            .done => unreachable,
        };
    }
}

// ── States ────────────────────────────────────────────────────────────────────
fn stateOpenRepo(machine: *TransactionMachine) CommitError!TransactionState {
    const gfile = c_libs.g_file_new_for_path(machine.commit.data.repo_path);
    defer c_libs.g_object_unref(gfile);

    const repo = c_libs.ostree_repo_new(gfile);
    if (c_libs.ostree_repo_open(repo, machine.commit.cancellable, &machine.commit.gerror) == 0) {
        c_libs.g_object_unref(repo);
        return machine.stateFailed(CommitError.RepoOpenFailed);
    }
    machine.repo = repo;

    return .get_prev_commit;
}

fn stateGetPrevCommit(machine: *TransactionMachine) CommitError!TransactionState {
    const repo = machine.repo orelse return machine.stateFailed(CommitError.RepoOpenFailed);

    _ = c_libs.ostree_repo_resolve_rev(repo, machine.commit.data.branch, 1, &machine.previous_commit_checksum, null);

    return .open_transaction;
}

fn stateOpenTransaction(machine: *TransactionMachine) CommitError!TransactionState {
    const repo = machine.repo orelse return machine.stateFailed(CommitError.RepoOpenFailed);

    if (c_libs.ostree_repo_prepare_transaction(repo, null, machine.commit.cancellable, &machine.commit.gerror) == 0) return machine.stateFailed(CommitError.RepoTransactionFailed);

    return .scan_root;
}

fn stateScanRoot(machine: *TransactionMachine) CommitError!TransactionState {
    const repo = machine.repo orelse return machine.stateFailed(CommitError.RepoOpenFailed);
    const mtree = machine.mtree orelse return machine.stateFailed(CommitError.RepoTransactionFailed);

    if (c_libs.ostree_repo_write_dfd_to_mtree(repo, std.c.AT.FDCWD, machine.commit.data.root_path, mtree, null, machine.commit.cancellable, &machine.commit.gerror) == 0) return machine.stateFailed(CommitError.CommitFailed);

    return .write_mtree;
}

fn stateWriteMtree(machine: *TransactionMachine) CommitError!TransactionState {
    const repo = machine.repo orelse return machine.stateFailed(CommitError.RepoOpenFailed);
    const mtree = machine.mtree orelse return machine.stateFailed(CommitError.RepoTransactionFailed);

    if (c_libs.ostree_repo_write_mtree(repo, mtree, &machine.mtree_root, machine.commit.cancellable, &machine.commit.gerror) == 0) return machine.stateFailed(CommitError.CommitFailed);

    return .write_commit;
}

fn stateWriteCommit(machine: *TransactionMachine) CommitError!TransactionState {
    const repo = machine.repo orelse return machine.stateFailed(CommitError.RepoOpenFailed);
    const mtree_root = machine.mtree_root orelse return machine.stateFailed(CommitError.CommitFailed);

    if (c_libs.ostree_repo_write_commit(repo, machine.previous_commit_checksum, machine.commit.data.message, null, null, @ptrCast(mtree_root), @ptrCast(&machine.commit_checksum), machine.commit.cancellable, &machine.commit.gerror) == 0) return machine.stateFailed(CommitError.CommitFailed);

    return .set_ref;
}

fn stateSetRef(machine: *TransactionMachine) CommitError!TransactionState {
    const repo = machine.repo orelse return machine.stateFailed(CommitError.RepoOpenFailed);

    c_libs.ostree_repo_transaction_set_ref(repo, null, machine.commit.data.branch, machine.commit_checksum);

    return .close_transaction;
}

fn stateCloseTransaction(machine: *TransactionMachine) CommitError!TransactionState {
    const repo = machine.repo orelse return machine.stateFailed(CommitError.RepoOpenFailed);

    if (machine.mtree_root) |root| {
        c_libs.g_object_unref(root);
        machine.mtree_root = null;
    }

    if (machine.mtree) |mtree| {
        c_libs.g_object_unref(mtree);
        machine.mtree = null;
    }

    if (c_libs.ostree_repo_commit_transaction(repo, null, machine.commit.cancellable, &machine.commit.gerror) == 0) return machine.stateFailed(CommitError.RepoTransactionFailed);

    return .close_repo;
}

fn stateCloseRepo(machine: *TransactionMachine) TransactionState {
    if (machine.repo) |repo| {
        c_libs.g_object_unref(repo);
        machine.repo = null;
    }

    return .done;
}
