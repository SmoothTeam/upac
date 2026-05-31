const std = @import("std");

const c_libs = @import("c-libs");

const types = @import("upac-types");
const PREFIX = types.paths.prefix;
const CONFIG_DIR = types.paths.config_dir;
const DB_PATH = types.paths.db_path;
const DB_NAME = types.paths.db_name;

const FileEntry = types.FileEntry;

const uninstaller = @import("../uninstaller.zig");
const UninstallerMachine = uninstaller.UninstallerMachine;
const UninstallerError = uninstaller.UninstallerError;

const database = @import("upac-database");
const Database = database.Database;
const exists = database.packages.exists;
const packages_delete = database.packages.delete;
const list = database.files.list;
const files_delete = database.files.delete;

const utils = @import("utils.zig");
const removeEmptyDirs = utils.removeEmptyDirs;
const removeFromMtree = utils.removeFromMtree;

// ── TransactionState ──────────────────────────────────────────────────────────
const TransactionState = enum {
    open_repo,
    get_prev_commit,
    get_mtree,
    open_database,
    load_package_files,
    remove_package_files,
    remove_package_records,
    remove_empty_dirs,
    close_database,
    build_commit_subject,
    open_transaction,
    write_commit,
    set_ref,
    close_transaction,
    close_repo,
    done,
};

// ── TransactionMachine ────────────────────────────────────────────────────────
pub const TransactionMachine = struct {
    uninstaller: *UninstallerMachine,

    current_package_index: usize = 0,
    current_package_uuid: ?[16]u8 = null,
    current_package_files: ?[]FileEntry = null,

    base: ?Database = null,

    repo: ?*c_libs.OstreeRepo = null,
    mtree: ?*c_libs.OstreeMutableTree = null,

    previos_commit_checksum: [65:0]u8 = std.mem.zeroes([65:0]u8),

    commit_checksum: ?[*c]u8 = null,
    commit_subject: [:0]const u8 = "",

    fn stateFailed(self: *TransactionMachine, err: UninstallerError) UninstallerError {
        var abort_error: ?*c_libs.GError = null;
        defer if (abort_error) |abort_err| c_libs.g_error_free(abort_err);

        if (self.repo) |repo| {
            _ = c_libs.ostree_repo_abort_transaction(repo, self.uninstaller.cancellable, &abort_error);

            if (self.commit_checksum != null) {
                _ = c_libs.ostree_repo_set_ref_immediate(
                    repo,
                    null,
                    self.uninstaller.data.branch,
                    &self.previos_commit_checksum,
                    self.uninstaller.cancellable,
                    null,
                );
            }

            c_libs.g_object_unref(repo);
            self.repo = null;
        }

        if (self.current_package_files) |package_files| {
            for (package_files) |*file_entry| file_entry.deinit(self.uninstaller.allocator);
            self.uninstaller.allocator.free(package_files);
            self.current_package_files = null;
        }

        if (self.base) |*base| {
            base.close();
            self.base = null;
        }

        if (self.commit_checksum) |checksum| {
            c_libs.g_free(checksum);
            self.commit_checksum = null;
        }
        if (self.commit_subject.len > 0) {
            self.uninstaller.allocator.free(self.commit_subject);
            self.commit_subject = "";
        }

        return err;
    }
};

// ── Trampoline ────────────────────────────────────────────────────────────────
pub fn run(machine: *UninstallerMachine) UninstallerError!void {
    var transaction_machine = TransactionMachine{ .uninstaller = machine };

    var state = TransactionState.open_repo;
    while (state != .done) {
        if (machine.cancellable) |cancellable| if (c_libs.g_cancellable_is_cancelled(cancellable) != 0) return transaction_machine.stateFailed(UninstallerError.Cancelled);

        state = switch (state) {
            .open_repo => try stateOpenRepo(&transaction_machine),
            .get_prev_commit => try stateGetPrevCommit(&transaction_machine),
            .get_mtree => try stateGetMtree(&transaction_machine),
            .open_database => try stateOpenDatabase(&transaction_machine),
            .load_package_files => try stateLoadPackageFiles(&transaction_machine),
            .remove_package_files => try stateRemovePackageFiles(&transaction_machine),
            .remove_package_records => try stateRemovePackageRecords(&transaction_machine),
            .remove_empty_dirs => try stateRemoveEmptyDirs(&transaction_machine),
            .close_database => stateCloseDatabase(&transaction_machine),
            .build_commit_subject => try stateBuildCommitSubject(&transaction_machine),
            .open_transaction => try stateOpenTransaction(&transaction_machine),
            .write_commit => try stateWriteCommit(&transaction_machine),
            .set_ref => try stateSetRef(&transaction_machine),
            .close_transaction => try stateCloseTransaction(&transaction_machine),
            .close_repo => stateCloseRepo(&transaction_machine),
            .done => unreachable,
        };
    }
}

// ── States ────────────────────────────────────────────────────────────────────
fn stateOpenRepo(machine: *TransactionMachine) UninstallerError!TransactionState {
    const gfile = c_libs.g_file_new_for_path(machine.uninstaller.data.repo_path);
    defer c_libs.g_object_unref(gfile);

    const repo = c_libs.ostree_repo_new(gfile);
    if (c_libs.ostree_repo_open(repo, machine.uninstaller.cancellable, &machine.uninstaller.gerror) == 0) {
        c_libs.g_object_unref(repo);
        return machine.stateFailed(UninstallerError.RepoOpenFailed);
    }
    machine.repo = repo;

    return .get_prev_commit;
}

fn stateGetPrevCommit(machine: *TransactionMachine) UninstallerError!TransactionState {
    const repo = machine.repo orelse return machine.stateFailed(UninstallerError.RepoOpenFailed);

    var checksum_ptr: [*c]u8 = null;
    defer c_libs.g_free(checksum_ptr);

    if (c_libs.ostree_repo_resolve_rev(repo, machine.uninstaller.data.branch, 0, &checksum_ptr, &machine.uninstaller.gerror) == 0) return machine.stateFailed(UninstallerError.RepoTransactionFailed);
    if (checksum_ptr == null) return machine.stateFailed(UninstallerError.CommitNotFound);

    const len = std.mem.len(checksum_ptr);
    @memcpy(machine.previos_commit_checksum[0..len], checksum_ptr[0..len]);
    machine.previos_commit_checksum[len] = 0;

    return .get_mtree;
}

fn stateGetMtree(machine: *TransactionMachine) UninstallerError!TransactionState {
    const repo = machine.repo orelse return machine.stateFailed(UninstallerError.RepoOpenFailed);
    const mtree = c_libs.ostree_mutable_tree_new_from_commit(repo, &machine.previos_commit_checksum, &machine.uninstaller.gerror) orelse return machine.stateFailed(UninstallerError.RepoOpenFailed);
    machine.mtree = mtree;

    return .open_database;
}

fn stateOpenDatabase(machine: *TransactionMachine) UninstallerError!TransactionState {
    const root_path = std.mem.span(machine.uninstaller.data.root_path);

    const database_file_path = std.fs.path.joinZ(machine.uninstaller.allocator, &.{ root_path, PREFIX, DB_PATH, DB_NAME }) catch return machine.stateFailed(UninstallerError.AllocZFailed);
    defer machine.uninstaller.allocator.free(database_file_path);

    machine.base = Database.open(machine.uninstaller.allocator, database_file_path) catch return machine.stateFailed(UninstallerError.ReadDatabaseFailed);

    return .load_package_files;
}

fn stateLoadPackageFiles(machine: *TransactionMachine) UninstallerError!TransactionState {
    const base = machine.base orelse return machine.stateFailed(UninstallerError.ReadDatabaseFailed);
    const package = machine.uninstaller.data.packages[machine.current_package_index];

    const uuid = exists(base, package.name, package.arch, package.arch_sub) catch return machine.stateFailed(UninstallerError.ReadDatabaseFailed);
    machine.current_package_uuid = uuid orelse return machine.stateFailed(UninstallerError.PackageNotFound);

    machine.current_package_files = list(base, machine.current_package_uuid.?) catch return machine.stateFailed(UninstallerError.ReadDatabaseFailed);

    return .remove_package_files;
}

fn stateRemovePackageFiles(machine: *TransactionMachine) UninstallerError!TransactionState {
    const package_files = machine.current_package_files orelse return machine.stateFailed(UninstallerError.FileMapCorrupted);

    for (package_files) |file_entry| {
        removeFromMtree(machine, file_entry.path) catch |err| {
            if (err == error.FileNotFound and std.mem.startsWith(u8, file_entry.path, CONFIG_DIR ++ "/")) continue;
            return machine.stateFailed(UninstallerError.FileMapCorrupted);
        };
    }

    return .remove_package_records;
}

fn stateRemovePackageRecords(machine: *TransactionMachine) UninstallerError!TransactionState {
    const base = machine.base orelse return machine.stateFailed(UninstallerError.ReadDatabaseFailed);
    const uuid = machine.current_package_uuid orelse return machine.stateFailed(UninstallerError.PackageNotFound);
    const package = machine.uninstaller.data.packages[machine.current_package_index];

    const package_files = machine.current_package_files orelse return machine.stateFailed(UninstallerError.FileMapCorrupted);
    for (package_files) |file_entry| {
        files_delete(base, uuid, file_entry.path) catch return machine.stateFailed(UninstallerError.ReadDatabaseFailed);
    }

    packages_delete(base, package.name, package.arch, package.arch_sub) catch return machine.stateFailed(UninstallerError.ReadDatabaseFailed);

    for (package_files) |*file_entry| file_entry.deinit(machine.uninstaller.allocator);
    machine.uninstaller.allocator.free(package_files);
    machine.current_package_files = null;
    machine.current_package_uuid = null;

    machine.current_package_index += 1;
    if (machine.current_package_index < machine.uninstaller.data.packages.len) return .load_package_files;

    machine.current_package_index = 0;
    return .remove_empty_dirs;
}

fn stateRemoveEmptyDirs(machine: *TransactionMachine) UninstallerError!TransactionState {
    const mtree = machine.mtree orelse return machine.stateFailed(UninstallerError.RepoOpenFailed);

    removeEmptyDirs(mtree, machine.uninstaller.allocator) catch return machine.stateFailed(UninstallerError.AllocZFailed);

    return .close_database;
}

fn stateCloseDatabase(machine: *TransactionMachine) TransactionState {
    if (machine.base) |*base| {
        base.close();
        machine.base = null;
    }

    return .build_commit_subject;
}

fn stateBuildCommitSubject(machine: *TransactionMachine) UninstallerError!TransactionState {
    var commit_subject_buf = std.Io.Writer.Allocating.init(machine.uninstaller.allocator);
    defer commit_subject_buf.deinit();

    commit_subject_buf.writer.writeAll("remove:") catch return machine.stateFailed(UninstallerError.AllocZFailed);
    for (machine.uninstaller.data.packages, 0..) |package, index|
        commit_subject_buf.writer.print("{s}{s}", .{ if (index == 0) " " else ", ", package.name }) catch return machine.stateFailed(UninstallerError.AllocZFailed);

    machine.commit_subject = machine.uninstaller.allocator.dupeZ(u8, commit_subject_buf.written()) catch return machine.stateFailed(UninstallerError.AllocZFailed);

    return .open_transaction;
}

fn stateOpenTransaction(machine: *TransactionMachine) UninstallerError!TransactionState {
    const repo = machine.repo orelse return machine.stateFailed(UninstallerError.RepoOpenFailed);

    if (c_libs.ostree_repo_prepare_transaction(repo, null, machine.uninstaller.cancellable, &machine.uninstaller.gerror) == 0) return machine.stateFailed(UninstallerError.RepoTransactionFailed);

    return .write_commit;
}

fn stateWriteCommit(machine: *TransactionMachine) UninstallerError!TransactionState {
    var new_checksum: [*c]u8 = null;
    var out_file: ?*c_libs.GFile = null;
    defer if (out_file) |file| c_libs.g_object_unref(file);

    const repo = machine.repo orelse return machine.stateFailed(UninstallerError.RepoOpenFailed);
    const mtree = machine.mtree orelse return machine.stateFailed(UninstallerError.RepoOpenFailed);

    if (c_libs.ostree_repo_write_mtree(repo, mtree, &out_file, machine.uninstaller.cancellable, &machine.uninstaller.gerror) == 0) return machine.stateFailed(UninstallerError.RepoTransactionFailed);

    if (c_libs.ostree_repo_write_commit(
        repo,
        &machine.previos_commit_checksum,
        machine.commit_subject.ptr,
        null,
        null,
        @as(?*c_libs.OstreeRepoFile, @ptrCast(out_file)),
        &new_checksum,
        machine.uninstaller.cancellable,
        &machine.uninstaller.gerror,
    ) == 0) return machine.stateFailed(UninstallerError.RepoTransactionFailed);

    machine.commit_checksum = new_checksum;

    return .set_ref;
}

fn stateSetRef(machine: *TransactionMachine) UninstallerError!TransactionState {
    const repo = machine.repo orelse return machine.stateFailed(UninstallerError.RepoOpenFailed);
    const checksum = machine.commit_checksum orelse return machine.stateFailed(UninstallerError.RepoTransactionFailed);

    c_libs.ostree_repo_transaction_set_ref(repo, null, machine.uninstaller.data.branch, checksum);

    return .close_transaction;
}

fn stateCloseTransaction(machine: *TransactionMachine) UninstallerError!TransactionState {
    const repo = machine.repo orelse return machine.stateFailed(UninstallerError.RepoOpenFailed);

    if (machine.commit_subject.len > 0) {
        machine.uninstaller.allocator.free(machine.commit_subject);
        machine.commit_subject = "";
    }

    if (c_libs.ostree_repo_commit_transaction(repo, null, machine.uninstaller.cancellable, &machine.uninstaller.gerror) == 0) return machine.stateFailed(UninstallerError.RepoTransactionFailed);

    return .close_repo;
}

fn stateCloseRepo(machine: *TransactionMachine) TransactionState {
    if (machine.repo) |repo| {
        c_libs.g_object_unref(repo);
        machine.repo = null;
    }

    return .done;
}
