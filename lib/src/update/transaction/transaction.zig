const std = @import("std");

const c_libs = @import("c-libs");

const types = @import("upac-types");
const PREFIX = types.paths.prefix;

const update = @import("../update.zig");
const UpdateMachine = update.UpdateMachine;
const UpdateError = update.UpdateError;

const utils = @import("utils.zig");
const removeFromMtree = utils.removeFromMtree;
const removeEmptyDirs = utils.removeEmptyDirs;
const formatVersion = utils.formatVersion;

// ── TransactionState ──────────────────────────────────────────────────────────
const TransactionState = enum {
    open_repo,
    get_prev_commit,
    get_mtree,
    open_transaction,
    write_package,
    remove_deleted_files,
    remove_empty_dirs,
    write_mtree,
    write_db_mtree,
    build_subject,
    write_commit,
    close_transaction,
    close_repo,
    done,
};

// ── TransactionMachine ────────────────────────────────────────────────────────
pub const TransactionMachine = struct {
    updater: *UpdateMachine,

    repo: ?*c_libs.OstreeRepo = null,
    mtree: ?*c_libs.OstreeMutableTree = null,
    mtree_root: ?*c_libs.GFile = null,

    previous_commit_checksum: [65:0]u8 = std.mem.zeroes([65:0]u8),
    commit_checksum: ?[*c]u8 = null,
    commit_subject: [:0]const u8 = "",

    current_package_index: usize = 0,

    fn stateFailed(self: *TransactionMachine, err: UpdateError) UpdateError {
        var abort_error: ?*c_libs.GError = null;
        defer if (abort_error) |abort_err| c_libs.g_error_free(abort_err);

        if (self.repo) |repo| {
            _ = c_libs.ostree_repo_abort_transaction(repo, null, &abort_error);

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

        if (self.commit_checksum) |checksum| {
            c_libs.g_free(checksum);
            self.commit_checksum = null;
        }

        if (self.commit_subject.len > 0) {
            self.updater.allocator.free(self.commit_subject);
            self.commit_subject = "";
        }

        return err;
    }
};

// ── Trampoline ────────────────────────────────────────────────────────────────
pub fn run(machine: *UpdateMachine) UpdateError!void {
    var transaction_machine = TransactionMachine{ .updater = machine };

    var state = TransactionState.open_repo;
    while (state != .done) {
        if (machine.cancellable) |cancellable| if (c_libs.g_cancellable_is_cancelled(cancellable) != 0) return transaction_machine.stateFailed(UpdateError.Cancelled);

        state = switch (state) {
            .open_repo => try stateOpenRepo(&transaction_machine),
            .get_prev_commit => try stateGetPrevCommit(&transaction_machine),
            .get_mtree => try stateGetMtree(&transaction_machine),
            .open_transaction => try stateOpenTransaction(&transaction_machine),
            .write_package => try stateWritePackage(&transaction_machine),
            .remove_deleted_files => try stateRemoveDeletedFiles(&transaction_machine),
            .remove_empty_dirs => try stateRemoveEmptyDirs(&transaction_machine),
            .write_mtree => try stateWriteMtree(&transaction_machine),
            .write_db_mtree => try stateWriteDbMtree(&transaction_machine),
            .build_subject => try stateBuildSubject(&transaction_machine),
            .write_commit => try stateWriteCommit(&transaction_machine),
            .close_transaction => try stateCloseTransaction(&transaction_machine),
            .close_repo => stateCloseRepo(&transaction_machine),
            .done => unreachable,
        };
    }
}

// ── States ────────────────────────────────────────────────────────────────────
fn stateOpenRepo(machine: *TransactionMachine) UpdateError!TransactionState {
    const gfile = c_libs.g_file_new_for_path(machine.updater.data.repo_path);
    defer c_libs.g_object_unref(gfile);

    const repo = c_libs.ostree_repo_new(gfile);
    if (c_libs.ostree_repo_open(repo, machine.updater.cancellable, &machine.updater.gerror) == 0) {
        c_libs.g_object_unref(repo);
        return machine.stateFailed(UpdateError.RepoOpenFailed);
    }
    machine.repo = repo;

    return .get_prev_commit;
}

fn stateGetPrevCommit(machine: *TransactionMachine) UpdateError!TransactionState {
    var checksum: [*c]u8 = null;
    defer c_libs.g_free(checksum);

    const repo = machine.repo orelse return machine.stateFailed(UpdateError.RepoOpenFailed);

    if (c_libs.ostree_repo_resolve_rev(repo, machine.updater.data.branch, 0, &checksum, &machine.updater.gerror) == 0) return machine.stateFailed(UpdateError.RepoTransactionFailed);
    if (checksum == null) return machine.stateFailed(UpdateError.CommitNotFound);

    const len = std.mem.len(checksum);
    @memcpy(machine.previous_commit_checksum[0..len], checksum[0..len]);
    machine.previous_commit_checksum[len] = 0;

    return .get_mtree;
}

fn stateGetMtree(machine: *TransactionMachine) UpdateError!TransactionState {
    const repo = machine.repo orelse return machine.stateFailed(UpdateError.RepoOpenFailed);

    const mtree = c_libs.ostree_mutable_tree_new_from_commit(repo, &machine.previous_commit_checksum, &machine.updater.gerror) orelse return machine.stateFailed(UpdateError.CommitNotFound);
    machine.mtree = mtree;

    return .open_transaction;
}

fn stateOpenTransaction(machine: *TransactionMachine) UpdateError!TransactionState {
    const repo = machine.repo orelse return machine.stateFailed(UpdateError.RepoOpenFailed);

    if (c_libs.ostree_repo_prepare_transaction(repo, null, machine.updater.cancellable, &machine.updater.gerror) == 0) {
        if (machine.updater.gerror) |err| {
            if (err.domain == c_libs.g_io_error_quark() and
                (err.code == c_libs.G_IO_ERROR_PERMISSION_DENIED or err.code == c_libs.G_IO_ERROR_READ_ONLY))
                return machine.stateFailed(UpdateError.AccessDenied);
        }
        return machine.stateFailed(UpdateError.RepoTransactionFailed);
    }

    return .write_package;
}

fn stateWritePackage(machine: *TransactionMachine) UpdateError!TransactionState {
    const repo = machine.repo orelse return machine.stateFailed(UpdateError.RepoOpenFailed);
    const mtree = machine.mtree orelse return machine.stateFailed(UpdateError.RepoOpenFailed);

    const package = machine.updater.data.packages[machine.current_package_index];

    const package_prefix_path = std.fs.path.joinZ(machine.updater.allocator, &.{ std.mem.span(package.temp_package_path), PREFIX }) catch return machine.stateFailed(UpdateError.AllocZFailed);
    defer machine.updater.allocator.free(package_prefix_path);

    if (c_libs.ostree_repo_write_dfd_to_mtree(repo, std.c.AT.FDCWD, package_prefix_path, mtree, null, machine.updater.cancellable, &machine.updater.gerror) == 0) return machine.stateFailed(UpdateError.WriteFilesFailed);

    machine.current_package_index += 1;
    if (machine.current_package_index < machine.updater.data.packages.len) return .write_package;

    machine.current_package_index = 0;
    return .remove_deleted_files;
}

fn stateRemoveDeletedFiles(machine: *TransactionMachine) UpdateError!TransactionState {
    const paths = machine.updater.deleted_file_paths orelse return .remove_empty_dirs;

    for (paths) |path| removeFromMtree(machine, path) catch |err| {
        if (err == error.FileNotFound) continue;
        return machine.stateFailed(UpdateError.WriteFilesFailed);
    };

    return .remove_empty_dirs;
}

fn stateRemoveEmptyDirs(machine: *TransactionMachine) UpdateError!TransactionState {
    const mtree = machine.mtree orelse return machine.stateFailed(UpdateError.RepoOpenFailed);

    removeEmptyDirs(mtree, machine.updater.allocator) catch return machine.stateFailed(UpdateError.AllocZFailed);

    return .write_db_mtree;
}

fn stateWriteMtree(machine: *TransactionMachine) UpdateError!TransactionState {
    const repo = machine.repo orelse return machine.stateFailed(UpdateError.RepoOpenFailed);
    const mtree = machine.mtree orelse return machine.stateFailed(UpdateError.RepoOpenFailed);

    if (c_libs.ostree_repo_write_mtree(repo, mtree, &machine.mtree_root, machine.updater.cancellable, &machine.updater.gerror) == 0) return machine.stateFailed(UpdateError.WriteFilesFailed);

    return .build_subject;
}

fn stateWriteDbMtree(machine: *TransactionMachine) UpdateError!TransactionState {
    const repo = machine.repo orelse return machine.stateFailed(UpdateError.RepoOpenFailed);
    const mtree = machine.mtree orelse return machine.stateFailed(UpdateError.RepoOpenFailed);

    const temp_database_path = machine.updater.temp_db_path orelse return machine.stateFailed(UpdateError.WriteDatabaseFailed);

    const temp_database_path_c = machine.updater.allocator.dupeZ(u8, std.mem.span(temp_database_path)) catch return machine.stateFailed(UpdateError.AllocZFailed);
    defer machine.updater.allocator.free(temp_database_path_c);

    if (c_libs.ostree_repo_write_dfd_to_mtree(repo, std.c.AT.FDCWD, temp_database_path_c, mtree, null, machine.updater.cancellable, &machine.updater.gerror) == 0) return machine.stateFailed(UpdateError.WriteFilesFailed);

    return .write_mtree;
}

fn stateBuildSubject(machine: *TransactionMachine) UpdateError!TransactionState {
    var subject_buf = std.Io.Writer.Allocating.init(machine.updater.allocator);
    defer subject_buf.deinit();

    subject_buf.writer.print("update:", .{}) catch return machine.stateFailed(UpdateError.AllocZFailed);

    for (machine.updater.data.packages, 0..) |package, index| {
        const separator: []const u8 = if (index == 0) " " else ", ";

        subject_buf.writer.print("{s}{s} ", .{ separator, package.meta.name }) catch return machine.stateFailed(UpdateError.AllocZFailed);

        formatVersion(package.meta.version, &subject_buf.writer) catch return machine.stateFailed(UpdateError.AllocZFailed);
    }

    machine.commit_subject = machine.updater.allocator.dupeZ(u8, subject_buf.written()) catch return machine.stateFailed(UpdateError.AllocZFailed);

    return .write_commit;
}

fn stateWriteCommit(machine: *TransactionMachine) UpdateError!TransactionState {
    var checksum: [*c]u8 = null;

    const repo = machine.repo orelse return machine.stateFailed(UpdateError.RepoOpenFailed);
    const mtree_root = machine.mtree_root orelse return machine.stateFailed(UpdateError.WriteFilesFailed);

    if (c_libs.ostree_repo_write_commit(repo, &machine.previous_commit_checksum, machine.commit_subject.ptr, null, null, @as(?*c_libs.OstreeRepoFile, @ptrCast(mtree_root)), &checksum, machine.updater.cancellable, &machine.updater.gerror) == 0) return machine.stateFailed(UpdateError.RepoTransactionFailed);

    const checksum_len = std.mem.len(checksum);
    @memcpy(machine.updater.new_commit_checksum[0..checksum_len], checksum[0..checksum_len]);
    c_libs.g_free(checksum);
    machine.commit_checksum = null;

    return .close_transaction;
}

fn stateCloseTransaction(machine: *TransactionMachine) UpdateError!TransactionState {
    const repo = machine.repo orelse return machine.stateFailed(UpdateError.RepoOpenFailed);

    if (machine.commit_subject.len > 0) {
        machine.updater.allocator.free(machine.commit_subject);
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

    if (c_libs.ostree_repo_commit_transaction(repo, null, machine.updater.cancellable, &machine.updater.gerror) == 0) {
        if (machine.updater.gerror) |err| {
            if (err.domain == c_libs.g_io_error_quark() and
                (err.code == c_libs.G_IO_ERROR_PERMISSION_DENIED or err.code == c_libs.G_IO_ERROR_READ_ONLY))
                return machine.stateFailed(UpdateError.AccessDenied);
        }
        return machine.stateFailed(UpdateError.RepoTransactionFailed);
    }

    return .close_repo;
}

fn stateCloseRepo(machine: *TransactionMachine) TransactionState {
    if (machine.repo) |repo| {
        c_libs.g_object_unref(repo);
        machine.repo = null;
    }

    return .done;
}
