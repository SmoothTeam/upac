const std = @import("std");

const c_libs = @import("c-libs");

const types = @import("upac-types");
const CONFIG_DIR = types.CONFIG_DIR;
const DB_RELATIVE_PATH = types.DB_RELATIVE_PATH;

const uninstaller = @import("../uninstaller.zig");
const UninstallerMachine = uninstaller.UninstallerMachine;
const UninstallerError = uninstaller.UninstallerError;

const database = @import("upac-database");
const FileMap = database.FileMap;
const freeFileMap = database.freeFileMap;
const readFiles = database.readFiles;

const find = @import("upac-index").find;

const utils = @import("utils.zig");
const loadCommitBody = utils.loadCommitBody;
const removeDbEntry = utils.removeDbEntry;
const removeEmptyDirs = utils.removeEmptyDirs;
const removeFromMtree = utils.removeFromMtree;

// ── TransactionState ──────────────────────────────────────────────────────────
const TransactionState = enum {
    open_repo,
    get_prev_commit,
    get_mtree,
    check_package_installed,
    load_package_files,
    remove_package_files,
    remove_empty_dirs,
    remove_package_db,
    build_commit_body,
    build_commit_subject,
    open_transaction,
    write_commit,
    set_ref,
    close_transaction,
    done,
};

// ── TransactionMachine ────────────────────────────────────────────────────────
pub const TransactionMachine = struct {
    uninstaller: *UninstallerMachine,

    current_package_index: usize = 0,
    current_package_checksum: ?[]const u8 = null,
    current_package_file_map: ?FileMap = null,

    repo: ?*c_libs.OstreeRepo = null,
    mtree: ?*c_libs.OstreeMutableTree = null,
    mtree_root: ?*c_libs.GFile = null,

    previos_commit_checksum: [65:0]u8 = std.mem.zeroes([65:0]u8),
    previos_commit_body: []const u8 = "",

    commit_checksum: ?[*c]u8 = null,
    commit_body: [:0]const u8 = "",
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
        }

        if (self.current_package_checksum) |checksum| {
            self.uninstaller.allocator.free(checksum);
            self.current_package_checksum = null;
        }
        if (self.previos_commit_body.len > 0) {
            self.uninstaller.allocator.free(self.previos_commit_body);
            self.previos_commit_body = "";
        }
        if (self.commit_checksum) |checksum| {
            c_libs.g_free(checksum);
            self.commit_checksum = null;
        }
        if (self.commit_body.len > 0) {
            self.uninstaller.allocator.free(self.commit_body);
            self.commit_body = "";
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

    if (machine.cancellable) |cancellable| {
        if (c_libs.g_cancellable_is_cancelled(cancellable) != 0) return transaction_machine.stateFailed(UninstallerError.Cancelled);
    }

    var state = TransactionState.open_repo;
    while (state != .done) {
        state = switch (state) {
            .open_repo => try stateOpenRepo(&transaction_machine),
            .get_prev_commit => try stateGetPrevCommit(&transaction_machine),
            .get_mtree => try stateGetMtree(&transaction_machine),
            .check_package_installed => try checkPackageInstalled(&transaction_machine),
            .load_package_files => try stateLoadPackageFiles(&transaction_machine),
            .remove_package_files => try stateRemovePackageFiles(&transaction_machine),
            .remove_empty_dirs => try stateRemoveEmptyDirs(&transaction_machine),
            .remove_package_db => try stateRemovePackageDb(&transaction_machine),
            .build_commit_body => try stateBuildCommitBody(&transaction_machine),
            .build_commit_subject => try stateBuildCommitSubject(&transaction_machine),
            .open_transaction => try stateOpenTransaction(&transaction_machine),
            .write_commit => try stateWriteCommit(&transaction_machine),
            .set_ref => try stateSetRef(&transaction_machine),
            .close_transaction => try stateCloseTransaction(&transaction_machine),
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

    machine.previos_commit_body = loadCommitBody(machine, checksum_ptr) catch |err| return machine.stateFailed(err);

    return .get_mtree;
}

fn stateGetMtree(machine: *TransactionMachine) UninstallerError!TransactionState {
    const repo = machine.repo orelse return machine.stateFailed(UninstallerError.RepoOpenFailed);
    const mtree = c_libs.ostree_mutable_tree_new_from_commit(repo, &machine.previos_commit_checksum, &machine.uninstaller.gerror) orelse return machine.stateFailed(UninstallerError.RepoOpenFailed);
    machine.mtree = mtree;

    return .check_package_installed;
}

fn checkPackageInstalled(machine: *TransactionMachine) UninstallerError!TransactionState {
    const package_name = machine.uninstaller.data.package_names[machine.current_package_index];

    const package_entry = find(machine.previos_commit_body, package_name, machine.uninstaller.allocator) catch return machine.stateFailed(UninstallerError.AllocZFailed);
    const found_package = package_entry orelse return machine.stateFailed(UninstallerError.PackageNotFound);

    machine.current_package_checksum = machine.uninstaller.allocator.dupe(u8, found_package.checksum) catch return machine.stateFailed(UninstallerError.AllocZFailed);

    return .load_package_files;
}

fn stateLoadPackageFiles(machine: *TransactionMachine) UninstallerError!TransactionState {
    const package_checksum = machine.current_package_checksum orelse return machine.stateFailed(UninstallerError.PackageNotFound);

    const package_database_path = std.fs.path.join(machine.uninstaller.allocator, &.{ std.mem.span(machine.uninstaller.data.root_path), DB_RELATIVE_PATH }) catch return machine.stateFailed(UninstallerError.AllocZFailed);
    defer machine.uninstaller.allocator.free(package_database_path);

    machine.current_package_file_map = readFiles(package_database_path, package_checksum, machine.uninstaller.allocator) catch return machine.stateFailed(UninstallerError.FileMapCorrupted);

    return .remove_package_files;
}

fn stateRemovePackageFiles(machine: *TransactionMachine) UninstallerError!TransactionState {
    const package_file_map = machine.current_package_file_map orelse return machine.stateFailed(UninstallerError.PackageNotFound);

    var pcakage_file_map_iter = package_file_map.iterator();
    while (pcakage_file_map_iter.next()) |package_file| {
        removeFromMtree(machine, package_file.key_ptr.*) catch |err| {
            if (err == error.FileNotFound and std.mem.startsWith(u8, package_file.key_ptr.*, CONFIG_DIR ++ "/")) continue;
            return machine.stateFailed(UninstallerError.FileMapCorrupted);
        };
    }

    return .remove_empty_dirs;
}

fn stateRemoveEmptyDirs(machine: *TransactionMachine) UninstallerError!TransactionState {
    const mtree = machine.mtree orelse return machine.stateFailed(UninstallerError.RepoOpenFailed);

    removeEmptyDirs(mtree, machine.uninstaller.allocator) catch return machine.stateFailed(UninstallerError.AllocZFailed);

    return .remove_package_db;
}

fn stateRemovePackageDb(machine: *TransactionMachine) UninstallerError!TransactionState {
    const checksum = machine.current_package_checksum orelse return machine.stateFailed(UninstallerError.PackageNotFound);

    removeDbEntry(machine, checksum, ".meta");
    removeDbEntry(machine, checksum, ".files");

    if (machine.current_package_file_map) |*file_map| {
        freeFileMap(file_map, machine.uninstaller.allocator);
        machine.current_package_file_map = null;
    }

    machine.uninstaller.allocator.free(checksum);
    machine.current_package_checksum = null;

    machine.current_package_index += 1;
    if (machine.current_package_index < machine.uninstaller.data.package_names.len) return .check_package_installed;

    machine.current_package_index = 0;
    return .build_commit_body;
}

fn stateBuildCommitBody(machine: *TransactionMachine) UninstallerError!TransactionState {
    var commit_body_buf = std.Io.Writer.Allocating.init(machine.uninstaller.allocator);
    defer commit_body_buf.deinit();

    var prevoios_commit_body_iter = std.mem.splitScalar(u8, machine.previos_commit_body, '\n');
    while (prevoios_commit_body_iter.next()) |line| {
        const trimmed_line = std.mem.trim(u8, line, " \t\r");
        if (trimmed_line.len == 0) continue;

        const separator_index = std.mem.indexOfScalar(u8, trimmed_line, ' ') orelse continue;
        const package_name = trimmed_line[0..separator_index];

        const should_remove = for (machine.uninstaller.data.package_names) |name| {
            if (std.ascii.eqlIgnoreCase(package_name, name)) break true;
        } else false;

        if (!should_remove) commit_body_buf.writer.print("{s}\n", .{trimmed_line}) catch return machine.stateFailed(UninstallerError.AllocZFailed);
    }

    machine.commit_body = machine.uninstaller.allocator.dupeZ(u8, commit_body_buf.written()) catch return machine.stateFailed(UninstallerError.AllocZFailed);

    return .build_commit_subject;
}

fn stateBuildCommitSubject(machine: *TransactionMachine) UninstallerError!TransactionState {
    var commit_subject_buf = std.Io.Writer.Allocating.init(machine.uninstaller.allocator);
    defer commit_subject_buf.deinit();

    commit_subject_buf.writer.writeAll("remove:") catch return machine.stateFailed(UninstallerError.AllocZFailed);
    for (machine.uninstaller.data.package_names, 0..) |name, index| commit_subject_buf.writer.print("{s}{s}", .{ if (index == 0) " " else ", ", name }) catch return machine.stateFailed(UninstallerError.AllocZFailed);

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
        machine.commit_body.ptr,
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
    if (machine.commit_body.len > 0) {
        machine.uninstaller.allocator.free(machine.commit_body);
        machine.commit_body = "";
    }

    if (c_libs.ostree_repo_commit_transaction(repo, null, machine.uninstaller.cancellable, &machine.uninstaller.gerror) == 0) return machine.stateFailed(UninstallerError.RepoTransactionFailed);

    return .done;
}
