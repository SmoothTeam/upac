const std = @import("std");

const c_libs = @import("c-libs");

const installer = @import("../installer.zig");
const InstallerMachine = installer.InstallerMachine;
const InstallerError = installer.InstallerError;

const utils = @import("utils.zig");
const formatVersion = utils.formatVersion;
// ── TransactionState ──────────────────────────────────────────────────────────
const TransactionState = enum {
    open_repo,
    get_prev_commit,
    get_mtree,
    open_transaction,
    write_package,
    write_mtree,
    write_db_mtree,
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
    while (state != .done) {
        if (machine.cancellable) |cancellable| if (c_libs.g_cancellable_is_cancelled(cancellable) != 0) return transaction_machine.stateFailed(InstallerError.Cancelled);

        state = switch (state) {
            .open_repo => try stateOpenRepo(&transaction_machine),
            .get_prev_commit => try stateGetPreviosCommit(&transaction_machine),
            .get_mtree => try stateGetPreviousMtree(&transaction_machine),
            .open_transaction => try stateOpenTransaction(&transaction_machine),
            .write_package => try stateWritePackageMtree(&transaction_machine),
            .write_mtree => try stateWriteGeneralMtree(&transaction_machine),
            .write_db_mtree => try stateWriteDbMtree(&transaction_machine),
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
    const package_path = package.temp_package_path;

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

    return .write_db_mtree;
}

fn stateWriteDbMtree(machine: *TransactionMachine) InstallerError!TransactionState {
    const repo = machine.repo orelse return machine.stateFailed(InstallerError.RepoOpenFailed);
    const mtree = machine.mtree orelse return machine.stateFailed(InstallerError.RepoOpenFailed);

    const temp_database_path = machine.installer.temp_db_path orelse return machine.stateFailed(InstallerError.WriteDatabaseFailed);

    const temp_database_path_c = machine.installer.allocator.dupeZ(u8, std.mem.span(temp_database_path)) catch return machine.stateFailed(InstallerError.AllocZFailed);
    defer machine.installer.allocator.free(temp_database_path_c);

    if (c_libs.ostree_repo_write_dfd_to_mtree(repo, std.c.AT.FDCWD, temp_database_path_c, mtree, null, machine.installer.cancellable, &machine.installer.gerror) == 0) return machine.stateFailed(InstallerError.WriteFilesFailed);

    return .build_subject;
}

fn stateBuildSubject(machine: *TransactionMachine) InstallerError!TransactionState {
    var subject_buf = std.Io.Writer.Allocating.init(machine.installer.allocator);
    defer subject_buf.deinit();

    subject_buf.writer.print("install:", .{}) catch return machine.stateFailed(InstallerError.AllocZFailed);

    for (machine.installer.data.packages, 0..) |package, index| {
        const separator: []const u8 = if (index == 0) " " else ", ";

        subject_buf.writer.print("{s}{s} ", .{ separator, package.meta.name }) catch return machine.stateFailed(InstallerError.AllocZFailed);

        formatVersion(package.meta.version, &subject_buf.writer) catch return machine.stateFailed(InstallerError.AllocZFailed);
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
        null,
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
