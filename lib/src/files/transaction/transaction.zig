const std = @import("std");

const c_libs = @import("c-libs");

const types = @import("upac-types");
const PREFIX = types.paths.prefix;
const DB_PATH = types.paths.db_path;
const DB_NAME = types.paths.db_name;

const database = @import("upac-database");
const Database = database.Database;

const files = @import("../files.zig");
const FilesMachine = files.FilesMachine;
const FilesError = files.FilesError;

const utils = @import("utils.zig");
const computeFileChecksum = utils.computeFileChecksum;
const addToMtree = utils.addToMtree;
const removeFromMtree = utils.removeFromMtree;

// ── TransactionState ──────────────────────────────────────────────────────────
const TransactionState = enum {
    copy_database,
    open_database,
    lookup_package,
    database_insert,
    database_delete,
    database_update,
    close_database,
    open_repo,
    get_prev_commit,
    get_mtree,
    open_transaction,
    repo_change,
    write_database_mtree,
    build_commit_subject,
    write_commit,
    set_ref,
    close_transaction,
    done,
};

// ── TransactionMachine ────────────────────────────────────────────────────────
pub const TransactionMachine = struct {
    files: *FilesMachine,

    current_file_index: usize = 0,

    base: ?Database = null,
    pkg_uuid: ?[16]u8 = null,

    repo: ?*c_libs.OstreeRepo = null,
    mtree: ?*c_libs.OstreeMutableTree = null,

    previous_commit_checksum: [65:0]u8 = std.mem.zeroes([65:0]u8),
    commit_checksum: ?[*c]u8 = null,
    commit_subject: [:0]const u8 = "",

    fn stateFailed(self: *TransactionMachine, err: FilesError) FilesError {
        var abort_error: ?*c_libs.GError = null;
        defer if (abort_error) |abort_err| c_libs.g_error_free(abort_err);

        if (self.repo) |repo| {
            _ = c_libs.ostree_repo_abort_transaction(repo, self.files.cancellable, &abort_error);

            if (self.commit_checksum != null) {
                _ = c_libs.ostree_repo_set_ref_immediate(repo, null, self.files.data.branch, &self.previous_commit_checksum, self.files.cancellable, null);
            }

            c_libs.g_object_unref(repo);
            self.repo = null;
        }

        if (self.mtree) |mtree| {
            c_libs.g_object_unref(mtree);
            self.mtree = null;
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
            self.files.allocator.free(self.commit_subject);
            self.commit_subject = "";
        }

        if (self.files.temp_database_path) |path| {
            const path_slice = std.mem.span(path);
            std.Io.Dir.cwd().deleteTree(self.files.io, path_slice) catch {};
            self.files.allocator.free(path_slice);
            self.files.temp_database_path = null;
        }

        return err;
    }
};

// ── Trampoline ────────────────────────────────────────────────────────────────
pub fn run(machine: *FilesMachine) FilesError!void {
    var transaction_machine = TransactionMachine{ .files = machine };

    var state = TransactionState.copy_database;
    while (state != .done) {
        if (machine.cancellable) |cancellable| if (c_libs.g_cancellable_is_cancelled(cancellable) != 0) return transaction_machine.stateFailed(FilesError.Cancelled);

        state = switch (state) {
            .copy_database => try stateCopyDatabase(&transaction_machine),
            .open_database => try stateOpenDatabase(&transaction_machine),
            .lookup_package => try stateLookupPackage(&transaction_machine),
            .database_insert => try stateDatabaseInsert(&transaction_machine),
            .database_delete => try stateDatabaseDelete(&transaction_machine),
            .database_update => try stateDatabaseUpdate(&transaction_machine),
            .close_database => stateCloseDatabase(&transaction_machine),
            .open_repo => try stateOpenRepo(&transaction_machine),
            .get_prev_commit => try stateGetPrevCommit(&transaction_machine),
            .get_mtree => try stateGetMtree(&transaction_machine),
            .open_transaction => try stateOpenTransaction(&transaction_machine),
            .repo_change => try stateRepoChange(&transaction_machine),
            .write_database_mtree => try stateWriteDatabaseMtree(&transaction_machine),
            .build_commit_subject => try stateBuildCommitSubject(&transaction_machine),
            .write_commit => try stateWriteCommit(&transaction_machine),
            .set_ref => try stateSetRef(&transaction_machine),
            .close_transaction => try stateCloseTransaction(&transaction_machine),
            .done => unreachable,
        };
    }
}

// ── States ────────────────────────────────────────────────────────────────────
fn stateCopyDatabase(machine: *TransactionMachine) FilesError!TransactionState {
    const timestamp: i64 = @intCast(@divTrunc(std.Io.Clock.real.now(machine.files.io).nanoseconds, std.time.ns_per_ms));

    const tmp_path = std.mem.span(machine.files.data.tmp_path);
    const root_path = std.mem.span(machine.files.data.root_path);

    const temp_name = std.fmt.allocPrint(machine.files.allocator, "upac-files-{d}", .{timestamp}) catch return machine.stateFailed(FilesError.AllocFailed);
    errdefer machine.files.allocator.free(temp_name);

    const temp_database_path = std.fs.path.joinZ(machine.files.allocator, &.{ tmp_path, temp_name }) catch return machine.stateFailed(FilesError.AllocFailed);
    machine.files.allocator.free(temp_name);

    machine.files.temp_database_path = temp_database_path;

    std.Io.Dir.cwd().createDirPath(machine.files.io, temp_database_path) catch return machine.stateFailed(FilesError.DatabaseWriteFailed);

    const source_db_path = std.fs.path.joinZ(machine.files.allocator, &.{ root_path, PREFIX, DB_PATH, DB_NAME }) catch return machine.stateFailed(FilesError.AllocFailed);
    defer machine.files.allocator.free(source_db_path);

    std.Io.Dir.copyFileAbsolute(source_db_path, temp_database_path, machine.files.io, .{}) catch return machine.stateFailed(FilesError.DatabaseWriteFailed);

    return .open_database;
}

fn stateOpenDatabase(machine: *TransactionMachine) FilesError!TransactionState {
    const temp_database_path = machine.files.temp_database_path orelse return machine.stateFailed(FilesError.DatabaseWriteFailed);

    machine.base = Database.open(machine.files.allocator, temp_database_path) catch return machine.stateFailed(FilesError.DatabaseWriteFailed);

    return .lookup_package;
}

fn stateLookupPackage(machine: *TransactionMachine) FilesError!TransactionState {
    const base = machine.base orelse return machine.stateFailed(FilesError.DatabaseReadFailed);

    const package_name = std.mem.span(machine.files.data.pkg_name);
    const package_arch = std.mem.span(machine.files.data.pkg_arch);
    const package_arch_sub = if (machine.files.data.pkg_arch_sub) |sub| std.mem.span(sub) else null;

    const uuid = database.packages.exists(base, package_name, package_arch, package_arch_sub) catch return machine.stateFailed(FilesError.DatabaseReadFailed);
    machine.pkg_uuid = uuid orelse return machine.stateFailed(FilesError.PackageNotFound);

    return switch (machine.files.data.kind) {
        .added => .database_insert,
        .removed => .database_delete,
        .modified => .database_update,
    };
}

fn stateDatabaseInsert(machine: *TransactionMachine) FilesError!TransactionState {
    const base = machine.base orelse return machine.stateFailed(FilesError.DatabaseReadFailed);
    const uuid = machine.pkg_uuid orelse return machine.stateFailed(FilesError.PackageNotFound);

    const current_file = machine.files.data.file_paths[machine.current_file_index];
    const path_absolute = std.mem.span(current_file);

    const sha256 = computeFileChecksum(machine, current_file) catch return machine.stateFailed(FilesError.DatabaseWriteFailed);

    const path_dupe = machine.files.allocator.dupe(u8, path_absolute) catch return machine.stateFailed(FilesError.AllocFailed);
    defer machine.files.allocator.free(path_dupe);

    database.files.insert(base, uuid, .{ .path = path_dupe, .sha256 = sha256, .is_user = true }) catch return machine.stateFailed(FilesError.DatabaseWriteFailed);

    machine.current_file_index += 1;
    if (machine.current_file_index < machine.files.data.file_paths.len) return .database_insert;
    return .close_database;
}

fn stateDatabaseDelete(machine: *TransactionMachine) FilesError!TransactionState {
    const base = machine.base orelse return machine.stateFailed(FilesError.DatabaseReadFailed);
    const uuid = machine.pkg_uuid orelse return machine.stateFailed(FilesError.PackageNotFound);

    const path_absolute = std.mem.span(machine.files.data.file_paths[machine.current_file_index]);

    database.files.delete(base, uuid, path_absolute) catch return machine.stateFailed(FilesError.DatabaseWriteFailed);

    machine.current_file_index += 1;
    if (machine.current_file_index < machine.files.data.file_paths.len) return .database_delete;
    return .close_database;
}

fn stateDatabaseUpdate(machine: *TransactionMachine) FilesError!TransactionState {
    const base = machine.base orelse return machine.stateFailed(FilesError.DatabaseReadFailed);
    const uuid = machine.pkg_uuid orelse return machine.stateFailed(FilesError.PackageNotFound);

    const current_file = machine.files.data.file_paths[machine.current_file_index];
    const path_absolute = std.mem.span(current_file);

    const sha256 = computeFileChecksum(machine, current_file) catch return machine.stateFailed(FilesError.DatabaseWriteFailed);

    const path_dupe = machine.files.allocator.dupe(u8, path_absolute) catch return machine.stateFailed(FilesError.AllocFailed);
    defer machine.files.allocator.free(path_dupe);

    database.files.update(base, uuid, .{ .path = path_dupe, .sha256 = sha256, .is_user = true }) catch return machine.stateFailed(FilesError.DatabaseWriteFailed);

    machine.current_file_index += 1;
    if (machine.current_file_index < machine.files.data.file_paths.len) return .database_update;
    return .close_database;
}

fn stateCloseDatabase(machine: *TransactionMachine) TransactionState {
    if (machine.base) |*base| {
        base.close();
        machine.base = null;
    }

    return .open_repo;
}

fn stateOpenRepo(machine: *TransactionMachine) FilesError!TransactionState {
    const gfile = c_libs.g_file_new_for_path(machine.files.data.repo_path);
    defer c_libs.g_object_unref(gfile);

    const repo = c_libs.ostree_repo_new(gfile);
    if (c_libs.ostree_repo_open(repo, machine.files.cancellable, &machine.files.gerror) == 0) {
        c_libs.g_object_unref(repo);
        return machine.stateFailed(FilesError.RepoOpenFailed);
    }
    machine.repo = repo;

    return .get_prev_commit;
}

fn stateGetPrevCommit(machine: *TransactionMachine) FilesError!TransactionState {
    const repo = machine.repo orelse return machine.stateFailed(FilesError.RepoOpenFailed);

    var checksum_ptr: [*c]u8 = null;
    defer c_libs.g_free(checksum_ptr);

    if (c_libs.ostree_repo_resolve_rev(repo, machine.files.data.branch, 0, &checksum_ptr, &machine.files.gerror) == 0) return machine.stateFailed(FilesError.RepoTransactionFailed);
    if (checksum_ptr == null) return machine.stateFailed(FilesError.RepoTransactionFailed);

    const len = std.mem.len(checksum_ptr);
    @memcpy(machine.previous_commit_checksum[0..len], checksum_ptr[0..len]);
    machine.previous_commit_checksum[len] = 0;

    return .get_mtree;
}

fn stateGetMtree(machine: *TransactionMachine) FilesError!TransactionState {
    const repo = machine.repo orelse return machine.stateFailed(FilesError.RepoOpenFailed);

    const mtree = c_libs.ostree_mutable_tree_new_from_commit(repo, &machine.previous_commit_checksum, &machine.files.gerror) orelse return machine.stateFailed(FilesError.RepoOpenFailed);
    machine.mtree = mtree;

    return .open_transaction;
}

fn stateOpenTransaction(machine: *TransactionMachine) FilesError!TransactionState {
    const repo = machine.repo orelse return machine.stateFailed(FilesError.RepoOpenFailed);

    if (c_libs.ostree_repo_prepare_transaction(repo, null, machine.files.cancellable, &machine.files.gerror) == 0) return machine.stateFailed(FilesError.RepoTransactionFailed);

    machine.current_file_index = 0;
    return .repo_change;
}

fn stateRepoChange(machine: *TransactionMachine) FilesError!TransactionState {
    const repo = machine.repo orelse return machine.stateFailed(FilesError.RepoOpenFailed);
    const mtree = machine.mtree orelse return machine.stateFailed(FilesError.RepoOpenFailed);

    switch (machine.files.data.kind) {
        .removed => removeFromMtree(machine, mtree) catch return machine.stateFailed(FilesError.RepoTransactionFailed),
        .added, .modified => addToMtree(machine, repo, mtree) catch return machine.stateFailed(FilesError.RepoTransactionFailed),
    }

    machine.current_file_index += 1;
    if (machine.current_file_index < machine.files.data.file_paths.len) return .repo_change;
    return .write_database_mtree;
}

fn stateWriteDatabaseMtree(machine: *TransactionMachine) FilesError!TransactionState {
    const repo = machine.repo orelse return machine.stateFailed(FilesError.RepoOpenFailed);
    const mtree = machine.mtree orelse return machine.stateFailed(FilesError.RepoOpenFailed);
    const temp_database_path = machine.files.temp_database_path orelse return machine.stateFailed(FilesError.DatabaseWriteFailed);

    const temp_databse_path_c = machine.files.allocator.dupeZ(u8, std.mem.span(temp_database_path)) catch return machine.stateFailed(FilesError.AllocFailed);
    defer machine.files.allocator.free(temp_databse_path_c);

    if (c_libs.ostree_repo_write_dfd_to_mtree(repo, std.c.AT.FDCWD, temp_databse_path_c, mtree, null, machine.files.cancellable, &machine.files.gerror) == 0) return machine.stateFailed(FilesError.RepoTransactionFailed);

    return .build_commit_subject;
}

fn stateBuildCommitSubject(machine: *TransactionMachine) FilesError!TransactionState {
    const pkg_name = std.mem.span(machine.files.data.pkg_name);
    const file_count = machine.files.data.file_paths.len;

    const prefix = switch (machine.files.data.kind) {
        .added => "files-add",
        .removed => "files-remove",
        .modified => "files-update",
    };

    var subject_buf = std.Io.Writer.Allocating.init(machine.files.allocator);
    defer subject_buf.deinit();

    subject_buf.writer.print("{s}: {d} file(s) ({s})", .{ prefix, file_count, pkg_name }) catch return machine.stateFailed(FilesError.AllocFailed);

    machine.commit_subject = machine.files.allocator.dupeZ(u8, subject_buf.written()) catch return machine.stateFailed(FilesError.AllocFailed);

    return .write_commit;
}

fn stateWriteCommit(machine: *TransactionMachine) FilesError!TransactionState {
    var new_checksum: [*c]u8 = null;
    var out_file: ?*c_libs.GFile = null;
    defer if (out_file) |file| c_libs.g_object_unref(file);

    const repo = machine.repo orelse return machine.stateFailed(FilesError.RepoOpenFailed);
    const mtree = machine.mtree orelse return machine.stateFailed(FilesError.RepoOpenFailed);

    if (c_libs.ostree_repo_write_mtree(repo, mtree, &out_file, machine.files.cancellable, &machine.files.gerror) == 0) return machine.stateFailed(FilesError.RepoTransactionFailed);

    if (c_libs.ostree_repo_write_commit(repo, &machine.previous_commit_checksum, machine.commit_subject.ptr, null, null, @as(?*c_libs.OstreeRepoFile, @ptrCast(out_file)), &new_checksum, machine.files.cancellable, &machine.files.gerror) == 0) return machine.stateFailed(FilesError.RepoTransactionFailed);

    machine.commit_checksum = new_checksum;

    return .set_ref;
}

fn stateSetRef(machine: *TransactionMachine) FilesError!TransactionState {
    const repo = machine.repo orelse return machine.stateFailed(FilesError.RepoOpenFailed);
    const checksum = machine.commit_checksum orelse return machine.stateFailed(FilesError.RepoTransactionFailed);

    c_libs.ostree_repo_transaction_set_ref(repo, null, machine.files.data.branch, checksum);

    return .close_transaction;
}

fn stateCloseTransaction(machine: *TransactionMachine) FilesError!TransactionState {
    const repo = machine.repo orelse return machine.stateFailed(FilesError.RepoOpenFailed);

    if (machine.commit_subject.len > 0) {
        machine.files.allocator.free(machine.commit_subject);
        machine.commit_subject = "";
    }

    if (machine.mtree) |mtree| {
        c_libs.g_object_unref(mtree);
        machine.mtree = null;
    }

    if (machine.commit_checksum) |checksum| {
        c_libs.g_free(checksum);
        machine.commit_checksum = null;
    }

    if (c_libs.ostree_repo_commit_transaction(repo, null, machine.files.cancellable, &machine.files.gerror) == 0) return machine.stateFailed(FilesError.RepoTransactionFailed);

    c_libs.g_object_unref(repo);
    machine.repo = null;

    return .done;
}
