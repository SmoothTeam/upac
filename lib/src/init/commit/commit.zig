const std = @import("std");

const c_libs = @import("c-libs");

const types = @import("upac-types");
const PREFIX = types.paths.prefix;

const init_module = @import("../init.zig");
const InitMachine = init_module.InitMachine;
const InitError = init_module.InitError;

// ── CommitState ───────────────────────────────────────────────────────────────
const CommitState = enum {
    open_repo,
    open_transaction,
    write_prefix,
    write_mtree,
    write_commit,
    set_ref,
    close_transaction,
    close_repo,
    done,
};

// ── CommitMachine ─────────────────────────────────────────────────────────────
const CommitMachine = struct {
    init: *InitMachine,

    repo: ?*c_libs.OstreeRepo = null,
    mtree: ?*c_libs.OstreeMutableTree = null,
    mtree_root: ?*c_libs.GFile = null,
    commit_checksum: [*c]u8 = null,

    fn stateFailed(self: *CommitMachine, err: InitError) InitError {
        if (self.commit_checksum != null) {
            c_libs.g_free(self.commit_checksum);
            self.commit_checksum = null;
        }
        if (self.mtree_root) |root| {
            c_libs.g_object_unref(root);
            self.mtree_root = null;
        }
        if (self.mtree) |mtree| {
            c_libs.g_object_unref(mtree);
            self.mtree = null;
        }
        if (self.repo) |repo| {
            var abort_error: ?*c_libs.GError = null;
            defer if (abort_error) |e| c_libs.g_error_free(e);
            _ = c_libs.ostree_repo_abort_transaction(repo, null, &abort_error);
            c_libs.g_object_unref(repo);
            self.repo = null;
        }
        return err;
    }
};

// ── Trampoline ────────────────────────────────────────────────────────────────
pub fn run(machine: *InitMachine) InitError!void {
    var commit_machine = CommitMachine{
        .init = machine,
        .mtree = c_libs.ostree_mutable_tree_new(),
    };

    var state = CommitState.open_repo;
    while (state != .done) {
        if (machine.cancellable) |cancellable| if (c_libs.g_cancellable_is_cancelled(cancellable) != 0) return commit_machine.stateFailed(InitError.Cancelled);

        state = switch (state) {
            .open_repo => try stateOpenRepo(&commit_machine),
            .open_transaction => try stateOpenTransaction(&commit_machine),
            .write_prefix => try stateWritePrefix(&commit_machine),
            .write_mtree => try stateWriteMtree(&commit_machine),
            .write_commit => try stateWriteCommit(&commit_machine),
            .set_ref => stateSetRef(&commit_machine),
            .close_transaction => try stateCloseTransaction(&commit_machine),
            .close_repo => stateCloseRepo(&commit_machine),
            .done => unreachable,
        };
    }
}

// ── States ────────────────────────────────────────────────────────────────────
fn stateOpenRepo(machine: *CommitMachine) InitError!CommitState {
    const gfile = c_libs.g_file_new_for_path(machine.init.data.repo_path);
    defer c_libs.g_object_unref(gfile);

    const repo = c_libs.ostree_repo_new(gfile);
    if (c_libs.ostree_repo_open(repo, machine.init.cancellable, &machine.init.gerror) == 0) {
        c_libs.g_object_unref(repo);
        return machine.stateFailed(InitError.OstreeInitFailed);
    }
    machine.repo = repo;

    return .open_transaction;
}

fn stateOpenTransaction(machine: *CommitMachine) InitError!CommitState {
    const repo = machine.repo orelse return machine.stateFailed(InitError.OstreeInitFailed);

    if (c_libs.ostree_repo_prepare_transaction(repo, null, machine.init.cancellable, &machine.init.gerror) == 0) return machine.stateFailed(InitError.OstreeCommitFailed);

    return .write_prefix;
}

fn stateWritePrefix(machine: *CommitMachine) InitError!CommitState {
    const repo = machine.repo orelse return machine.stateFailed(InitError.OstreeInitFailed);
    const mtree = machine.mtree orelse return machine.stateFailed(InitError.OstreeCommitFailed);

    const root_path = std.mem.span(machine.init.data.root_path);

    const prefix_path = std.fs.path.joinZ(machine.init.allocator, &.{ root_path, PREFIX }) catch return machine.stateFailed(InitError.AllocFailed);
    defer machine.init.allocator.free(prefix_path);

    if (c_libs.ostree_repo_write_dfd_to_mtree(repo, std.c.AT.FDCWD, prefix_path, mtree, null, machine.init.cancellable, &machine.init.gerror) == 0) return machine.stateFailed(InitError.OstreeCommitFailed);

    return .write_mtree;
}

fn stateWriteMtree(machine: *CommitMachine) InitError!CommitState {
    const repo = machine.repo orelse return machine.stateFailed(InitError.OstreeInitFailed);
    const mtree = machine.mtree orelse return machine.stateFailed(InitError.OstreeCommitFailed);

    if (c_libs.ostree_repo_write_mtree(repo, mtree, &machine.mtree_root, machine.init.cancellable, &machine.init.gerror) == 0) return machine.stateFailed(InitError.OstreeCommitFailed);

    c_libs.g_object_unref(mtree);
    machine.mtree = null;

    return .write_commit;
}

fn stateWriteCommit(machine: *CommitMachine) InitError!CommitState {
    const repo = machine.repo orelse return machine.stateFailed(InitError.OstreeInitFailed);
    const mtree_root = machine.mtree_root orelse return machine.stateFailed(InitError.OstreeCommitFailed);

    if (c_libs.ostree_repo_write_commit(
        repo,
        null,
        "init",
        null,
        null,
        @ptrCast(mtree_root),
        @ptrCast(&machine.commit_checksum),
        machine.init.cancellable,
        &machine.init.gerror,
    ) == 0) return machine.stateFailed(InitError.OstreeCommitFailed);

    c_libs.g_object_unref(mtree_root);
    machine.mtree_root = null;

    return .set_ref;
}

fn stateSetRef(machine: *CommitMachine) CommitState {
    const repo = machine.repo orelse return .close_repo;

    c_libs.ostree_repo_transaction_set_ref(repo, null, machine.init.data.branch, machine.commit_checksum);

    return .close_transaction;
}

fn stateCloseTransaction(machine: *CommitMachine) InitError!CommitState {
    const repo = machine.repo orelse return machine.stateFailed(InitError.OstreeInitFailed);

    if (machine.commit_checksum != null) {
        c_libs.g_free(machine.commit_checksum);
        machine.commit_checksum = null;
    }

    if (c_libs.ostree_repo_commit_transaction(repo, null, machine.init.cancellable, &machine.init.gerror) == 0) return machine.stateFailed(InitError.OstreeCommitFailed);

    return .close_repo;
}

fn stateCloseRepo(machine: *CommitMachine) CommitState {
    if (machine.repo) |repo| {
        c_libs.g_object_unref(repo);
        machine.repo = null;
    }

    return .done;
}
