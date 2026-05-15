const std = @import("std");

const installer = @import("../installer.zig");
const c_libs = installer.ffi.c_libs;

const InstallerMachine = installer.InstallerMachine;
const InstallerError = installer.InstallerError;

const append = installer.index.append;

const loadCommitBody = @import("utils.zig").loadCommitBody;
// ── TransactionState ──────────────────────────────────────────────────────────
const TransactionState = enum {
    open_repo,
    get_prev_commit,
    get_mtree,
    open_transaction,
    write_package,
    write_mtree,
    load_body,
    build_body,
    build_subject,
    write_commit,
    set_ref,
    close_transaction,
    done,
};

// ── TransactionMachine ────────────────────────────────────────────────────────
pub const TransactionMachine = struct {
    installer: *InstallerMachine,

    repo: ?*c_libs.OstreeRepo = null,
    mtree: ?*c_libs.OstreeMutableTree = null,
    mtree_root: ?*c_libs.GFile = null,

    previous_commit_checksum: [*c]u8 = null,
    commit_checksum: [*c]u8 = null,

    commit_body: []const u8 = "",
    commit_subject: [:0]const u8 = "",

    current_package_index: usize = 0,

    fn stateFailed(self: *TransactionMachine, err: InstallerError) InstallerError {
        var abort_error: ?*c_libs.GError = null;
        defer if (abort_error) |abort_err| c_libs.g_error_free(abort_err);

        if (self.repo) |repo| {
            _ = c_libs.ostree_repo_abort_transaction(repo, null, &abort_error);

            if (self.commit_checksum != null) _ = c_libs.ostree_repo_set_ref_immediate(repo, null, self.installer.data.branch, self.previous_commit_checksum, null, null);
        }

        if (self.mtree_root) |root| {
            c_libs.g_object_unref(root);
            self.mtree_root = null;
        }
        if (self.mtree) |mtree| {
            c_libs.g_object_unref(mtree);
            self.mtree = null;
        }
        if (self.commit_body.len > 0) self.installer.allocator.free(self.commit_body);
        if (self.commit_subject.len > 0) self.installer.allocator.free(self.commit_subject);

        return err;
    }
};

// ── Trampoline ────────────────────────────────────────────────────────────────
pub fn run(machine: *InstallerMachine) InstallerError!void {
    var transaction_machine = TransactionMachine{ .installer = machine, .mtree = c_libs.ostree_mutable_tree_new() };

    if (transaction_machine.repo) |repo| {
        var objects_total: c_libs.gint = 0;
        var objects_pruned: c_libs.gint = 0;
        var pruned_size: c_libs.guint64 = 0;
        _ = c_libs.ostree_repo_prune(repo, c_libs.OSTREE_REPO_PRUNE_FLAGS_REFS_ONLY, -1, &objects_total, &objects_pruned, &pruned_size, machine.cancellable, null);
    }

    var state = TransactionState.open_repo;
    if (machine.cancellable) |cancellable| {
        if (c_libs.g_cancellable_is_cancelled(cancellable) != 0) return transaction_machine.stateFailed(InstallerError.Cancelled);
    }

    while (state != .done) {
        state = switch (state) {
            .open_repo => try stateOpenRepo(&transaction_machine),
            .get_prev_commit => try stateGetPreviosCommit(&transaction_machine),
            .get_mtree => try stateGetPreviousMtree(&transaction_machine),
            .open_transaction => try stateOpenTransaction(&transaction_machine),
            .write_package => try stateWritePackageMtree(&transaction_machine),
            .write_mtree => try stateWriteGeneralMtree(&transaction_machine),
            .load_body => try stateLoadPreviosBody(&transaction_machine),
            .build_body => try stateBuildBody(&transaction_machine),
            .build_subject => try stateBuildSubject(&transaction_machine),
            .write_commit => try stateWriteCommit(&transaction_machine),
            .set_ref => try stateSetRef(&transaction_machine),
            .close_transaction => try stateCloseTransaction(&transaction_machine),
            .done => unreachable,
        };
    }
}

// ── States ────────────────────────────────────────────────────────────────────
fn stateOpenRepo(machine: *TransactionMachine) InstallerError!TransactionState {
    const gfile = c_libs.g_file_new_for_path(machine.installer.data.repo_path);
    defer c_libs.g_object_unref(gfile);

    const repo = c_libs.ostree_repo_new(gfile);
    if (c_libs.ostree_repo_open(repo, machine.installer.cancellable, &machine.installer.gerror) == 0) {
        c_libs.g_object_unref(repo);
        return machine.stateFailed(InstallerError.RepoOpenFailed);
    }
    machine.repo = repo;

    return .get_prev_commit;
}

fn stateGetPreviosCommit(machine: *TransactionMachine) InstallerError!TransactionState {
    const repo = machine.repo orelse return machine.stateFailed(InstallerError.RepoOpenFailed);

    _ = c_libs.ostree_repo_resolve_rev(repo, machine.installer.data.branch, 1, &machine.previous_commit_checksum, null);

    if (machine.previous_commit_checksum == null) return .open_transaction;
    return .get_mtree;
}

fn stateGetPreviousMtree(machine: *TransactionMachine) InstallerError!TransactionState {
    var previos_root: ?*c_libs.GFile = null;
    defer if (previos_root) |root| c_libs.g_object_unref(root);

    const repo = machine.repo orelse return machine.stateFailed(InstallerError.RepoOpenFailed);
    const previos_checksum = machine.previous_commit_checksum orelse return machine.stateFailed(InstallerError.CommitNotFound);

    if (c_libs.ostree_repo_read_commit(repo, previos_checksum, &previos_root, null, machine.installer.cancellable, &machine.installer.gerror) == 0) return machine.stateFailed(InstallerError.CommitNotFound);

    if (c_libs.ostree_repo_write_directory_to_mtree(repo, previos_root, machine.mtree, null, machine.installer.cancellable, &machine.installer.gerror) == 0) return machine.stateFailed(InstallerError.WriteFilesFailed);

    return .load_body;
}

fn stateLoadPreviosBody(machine: *TransactionMachine) InstallerError!TransactionState {
    const previos_checksum = machine.previous_commit_checksum orelse return machine.stateFailed(InstallerError.CommitNotFound);

    machine.commit_body = loadCommitBody(machine, previos_checksum) catch |err| return machine.stateFailed(err);

    return .open_transaction;
}

fn stateOpenTransaction(machine: *TransactionMachine) InstallerError!TransactionState {
    const repo = machine.repo orelse return machine.stateFailed(InstallerError.RepoOpenFailed);

    if (c_libs.ostree_repo_prepare_transaction(repo, null, machine.installer.cancellable, &machine.installer.gerror) == 0) return machine.stateFailed(InstallerError.RepoTransactionFailed);

    return .write_package;
}

fn stateWritePackageMtree(machine: *TransactionMachine) InstallerError!TransactionState {
    const repo = machine.repo orelse return machine.stateFailed(InstallerError.RepoOpenFailed);
    const mtree = machine.mtree orelse return machine.stateFailed(InstallerError.RepoOpenFailed);

    const package = machine.installer.data.packages[machine.current_package_index];
    const package_path = std.mem.span(package.path);

    if (c_libs.ostree_repo_write_dfd_to_mtree(repo, std.c.AT.FDCWD, package_path, mtree, null, machine.installer.cancellable, &machine.installer.gerror) == 0) return machine.stateFailed(InstallerError.WriteFilesFailed);

    machine.current_package_index += 1;
    if (machine.current_package_index < machine.installer.data.packages.len) return .write_package;

    machine.current_package_index = 0;
    return .write_mtree;
}

fn stateWriteGeneralMtree(machine: *TransactionMachine) InstallerError!TransactionState {
    const repo = machine.repo orelse return machine.stateFailed(InstallerError.RepoOpenFailed);
    const mtree = machine.mtree orelse return machine.stateFailed(InstallerError.RepoOpenFailed);

    if (c_libs.ostree_repo_write_mtree(repo, mtree, &machine.mtree_root, machine.installer.cancellable, &machine.installer.gerror) == 0) return machine.stateFailed(InstallerError.WriteFilesFailed);

    return .build_body;
}

fn stateBuildBody(machine: *TransactionMachine) InstallerError!TransactionState {
    const package = machine.installer.data.packages[machine.current_package_index];

    const new_body = append(machine.commit_body, package.meta.name, package.checksum, machine.installer.allocator) catch return machine.stateFailed(InstallerError.AllocZFailed);

    if (machine.commit_body.len > 0) machine.installer.allocator.free(machine.commit_body);
    machine.commit_body = new_body;

    machine.current_package_index += 1;
    if (machine.current_package_index < machine.installer.data.packages.len) return .build_body;

    machine.current_package_index = 0;
    return .build_subject;
}

fn stateBuildSubject(machine: *TransactionMachine) InstallerError!TransactionState {
    var subject_buf = std.Io.Writer.Allocating.init(machine.installer.allocator);
    defer subject_buf.deinit();

    const first_package = machine.installer.data.packages[0];
    subject_buf.writer.print("install: {s} {s}", .{ first_package.meta.name, first_package.meta.version }) catch return machine.stateFailed(InstallerError.AllocZFailed);

    for (machine.installer.data.packages[1..]) |package| {
        subject_buf.writer.print(", {s} {s}", .{ package.meta.name, package.meta.version }) catch return machine.stateFailed(InstallerError.AllocZFailed);
    }

    machine.commit_subject = machine.installer.allocator.dupeZ(u8, subject_buf.written()) catch return machine.stateFailed(InstallerError.AllocZFailed);

    return .write_commit;
}

fn stateWriteCommit(machine: *TransactionMachine) InstallerError!TransactionState {
    const repo = machine.repo orelse return machine.stateFailed(InstallerError.RepoOpenFailed);
    const mtree_root = machine.mtree_root orelse return machine.stateFailed(InstallerError.WriteFilesFailed);

    if (c_libs.ostree_repo_write_commit(
        repo,
        machine.previous_commit_checksum,
        machine.commit_subject,
        machine.commit_body.ptr,
        null,
        @ptrCast(mtree_root),
        @ptrCast(&machine.commit_checksum),
        machine.installer.cancellable,
        &machine.installer.gerror,
    ) == 0) return machine.stateFailed(InstallerError.RepoTransactionFailed);

    return .set_ref;
}

fn stateSetRef(machine: *TransactionMachine) InstallerError!TransactionState {
    const repo = machine.repo orelse return machine.stateFailed(InstallerError.RepoOpenFailed);

    c_libs.ostree_repo_transaction_set_ref(repo, null, machine.installer.data.branch, machine.commit_checksum);

    return .close_transaction;
}

fn stateCloseTransaction(machine: *TransactionMachine) InstallerError!TransactionState {
    const repo = machine.repo orelse return machine.stateFailed(InstallerError.RepoOpenFailed);

    if (machine.commit_subject.len > 0) {
        machine.installer.allocator.free(machine.commit_subject);
        machine.commit_subject = "";
    }
    if (machine.commit_body.len > 0) {
        machine.installer.allocator.free(machine.commit_body);
        machine.commit_body = "";
    }

    if (machine.mtree_root) |root| {
        c_libs.g_object_unref(root);
        machine.mtree_root = null;
    }
    if (machine.mtree) |mtree| {
        c_libs.g_object_unref(mtree);
        machine.mtree = null;
    }

    if (c_libs.ostree_repo_commit_transaction(repo, null, machine.installer.cancellable, &machine.installer.gerror) == 0) return machine.stateFailed(InstallerError.RepoTransactionFailed);

    return .done;
}
