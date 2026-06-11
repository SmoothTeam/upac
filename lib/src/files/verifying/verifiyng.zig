const std = @import("std");

const c_libs = @import("c-libs");

const types = @import("upac-types");
const PREFIX = types.paths.prefix;
const DB_PATH = types.paths.db_path;
const DB_NAME = types.paths.db_name;

const database = @import("upac-database");
const Database = database.Database;
const packages = database.packages;

const files = @import("../files.zig");
const FilesMachine = files.FilesMachine;
const FilesError = files.FilesError;

// ── VerifyingState ────────────────────────────────────────────────────────────
const VerifyingState = enum {
    check_root,
    check_prefix,
    check_database_file,
    check_file_path,
    check_file_exists,
    open_database,
    check_package,
    close_database,
    open_repo,
    check_branch,
    close_repo,
    done,
};

// ── VerifyingMachine ──────────────────────────────────────────────────────────
const VerifyingMachine = struct {
    files: *FilesMachine,

    current_file_index: usize = 0,

    base: ?Database = null,
    repo: ?*c_libs.OstreeRepo = null,

    fn stateFailed(self: *VerifyingMachine, err: FilesError) FilesError {
        if (self.base) |*base| {
            base.close();
            self.base = null;
        }

        if (self.repo) |repo| {
            c_libs.g_object_unref(repo);
            self.repo = null;
        }

        return err;
    }
};

// ── Trampoline ────────────────────────────────────────────────────────────────
pub fn run(machine: *FilesMachine) FilesError!void {
    var verifying_machine = VerifyingMachine{ .files = machine };

    var state = VerifyingState.check_root;
    while (state != .done) {
        if (machine.cancellable) |cancellable| if (c_libs.g_cancellable_is_cancelled(cancellable) != 0) return verifying_machine.stateFailed(FilesError.Cancelled);

        state = switch (state) {
            .check_root => try stateCheckRoot(&verifying_machine),
            .check_prefix => try stateCheckPrefix(&verifying_machine),
            .check_database_file => try stateCheckDatabaseFile(&verifying_machine),
            .check_file_path => try stateCheckFilePath(&verifying_machine),
            .check_file_exists => try stateCheckFileExists(&verifying_machine),
            .open_database => try stateOpenDatabase(&verifying_machine),
            .check_package => try stateCheckPackage(&verifying_machine),
            .close_database => stateCloseDatabase(&verifying_machine),
            .open_repo => try stateOpenRepo(&verifying_machine),
            .check_branch => try stateCheckBranch(&verifying_machine),
            .close_repo => stateCloseRepo(&verifying_machine),
            .done => unreachable,
        };
    }
}

// ── States ────────────────────────────────────────────────────────────────────
fn stateCheckRoot(machine: *VerifyingMachine) FilesError!VerifyingState {
    const root_path = std.mem.span(machine.files.data.root_path);

    std.Io.Dir.accessAbsolute(machine.files.io, root_path, .{}) catch return FilesError.PathNotFound;

    return .check_prefix;
}

fn stateCheckPrefix(machine: *VerifyingMachine) FilesError!VerifyingState {
    const root_path = std.mem.span(machine.files.data.root_path);

    const prefix_path = std.fs.path.joinZ(machine.files.allocator, &.{ root_path, PREFIX }) catch return FilesError.AllocFailed;
    defer machine.files.allocator.free(prefix_path);

    std.Io.Dir.accessAbsolute(machine.files.io, prefix_path, .{}) catch return FilesError.PathNotFound;

    return .check_database_file;
}

fn stateCheckDatabaseFile(machine: *VerifyingMachine) FilesError!VerifyingState {
    const root_path = std.mem.span(machine.files.data.root_path);

    const database_file_path = std.fs.path.joinZ(machine.files.allocator, &.{ root_path, PREFIX, DB_PATH, DB_NAME }) catch return FilesError.AllocFailed;
    defer machine.files.allocator.free(database_file_path);

    std.Io.Dir.accessAbsolute(machine.files.io, database_file_path, .{}) catch return FilesError.DatabaseNotFound;

    return .check_file_path;
}

fn stateCheckFilePath(machine: *VerifyingMachine) FilesError!VerifyingState {
    const repo_path = std.mem.span(machine.files.data.repo_path);
    const file_path = std.mem.span(machine.files.data.file_paths[machine.current_file_index]);

    if (std.mem.startsWith(u8, file_path, repo_path)) return FilesError.InvalidFilePath;

    return .check_file_exists;
}

fn stateCheckFileExists(machine: *VerifyingMachine) FilesError!VerifyingState {
    const file_path = std.mem.span(machine.files.data.file_paths[machine.current_file_index]);

    std.Io.Dir.accessAbsolute(machine.files.io, file_path, .{}) catch return FilesError.PathNotFound;

    machine.current_file_index += 1;
    if (machine.current_file_index < machine.files.data.file_paths.len) return .check_file_path;

    return .open_database;
}

fn stateOpenDatabase(machine: *VerifyingMachine) FilesError!VerifyingState {
    const root_path = std.mem.span(machine.files.data.root_path);

    const database_file_path = std.fs.path.joinZ(machine.files.allocator, &.{ root_path, PREFIX, DB_PATH, DB_NAME }) catch return machine.stateFailed(FilesError.AllocFailed);
    defer machine.files.allocator.free(database_file_path);

    machine.base = Database.open(machine.files.allocator, database_file_path, false) catch |err| return machine.stateFailed(switch (err) {
        error.AccessDenied => FilesError.AccessDenied,
        else => FilesError.DatabaseNotFound,
    });

    return .check_package;
}

fn stateCheckPackage(machine: *VerifyingMachine) FilesError!VerifyingState {
    const base = machine.base orelse return machine.stateFailed(FilesError.DatabaseReadFailed);

    const pkg_name = std.mem.span(machine.files.data.pkg_name);
    const pkg_arch = std.mem.span(machine.files.data.pkg_arch);
    const pkg_arch_sub = if (machine.files.data.pkg_arch_sub) |sub| std.mem.span(sub) else null;

    const uuid = packages.exists(base, pkg_name, pkg_arch, pkg_arch_sub) catch return machine.stateFailed(FilesError.DatabaseReadFailed);
    if (uuid == null) return machine.stateFailed(FilesError.PackageNotFound);

    return .close_database;
}

fn stateCloseDatabase(machine: *VerifyingMachine) VerifyingState {
    if (machine.base) |*base| {
        base.close();
        machine.base = null;
    }

    return .open_repo;
}

fn stateOpenRepo(machine: *VerifyingMachine) FilesError!VerifyingState {
    const gfile = c_libs.g_file_new_for_path(machine.files.data.repo_path);
    defer c_libs.g_object_unref(gfile);

    const repo = c_libs.ostree_repo_new(gfile);
    if (c_libs.ostree_repo_open(repo, machine.files.cancellable, &machine.files.gerror) == 0) {
        c_libs.g_object_unref(repo);
        return machine.stateFailed(FilesError.RepoOpenFailed);
    }
    machine.repo = repo;

    return .check_branch;
}

fn stateCheckBranch(machine: *VerifyingMachine) FilesError!VerifyingState {
    const repo = machine.repo orelse return machine.stateFailed(FilesError.RepoOpenFailed);

    var checksum: [*c]u8 = null;
    defer c_libs.g_free(checksum);

    if (c_libs.ostree_repo_resolve_rev(repo, machine.files.data.branch, 0, &checksum, &machine.files.gerror) == 0) return machine.stateFailed(FilesError.RepoTransactionFailed);
    if (checksum == null) return machine.stateFailed(FilesError.RepoTransactionFailed);

    return .close_repo;
}

fn stateCloseRepo(machine: *VerifyingMachine) VerifyingState {
    if (machine.repo) |repo| {
        c_libs.g_object_unref(repo);
        machine.repo = null;
    }

    return .done;
}
