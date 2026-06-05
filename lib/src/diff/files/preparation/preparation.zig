const std = @import("std");

const c_libs = @import("c-libs");

const types = @import("upac-types");
const DB_NAME = types.paths.db_name;

const DiffError = types.DiffError;

const database = @import("upac-database");
const Database = database.Database;

const diff = @import("../files.zig");
const DiffMachine = diff.DiffMachine;

const utils = @import("utils.zig");
const buildFilePkgMap = utils.buildFilePkgMap;
const checkoutDb = utils.checkoutDb;

// ── PreparationState ──────────────────────────────────────────────────────────
const PreparationState = enum {
    open_repo,
    checkout_database,
    load_package_file_map,
    cleanup_database,
    close_repo,
    done,
};

// ── PreparationMachine ────────────────────────────────────────────────────────
pub const PreparationMachine = struct {
    diff: *DiffMachine,

    repo: ?*c_libs.OstreeRepo = null,

    current_ref_index: usize = 0,
    current_database_path: ?[]u8 = null,

    fn currentRef(self: *PreparationMachine) [*:0]const u8 {
        return if (self.current_ref_index == 0) self.diff.data.from_ref else self.diff.data.to_ref;
    }

    fn stateFailed(self: *PreparationMachine, err: DiffError) DiffError {
        if (self.current_database_path) |path| {
            std.Io.Dir.cwd().deleteTree(self.diff.io, path) catch {};
            self.diff.allocator.free(path);
        }
        self.current_database_path = null;

        if (self.repo) |repo| {
            c_libs.g_object_unref(repo);
            self.repo = null;
        }

        return err;
    }
};

// ── Trampoline ────────────────────────────────────────────────────────────────
pub fn run(machine: *DiffMachine) DiffError!void {
    var preparation_machine = PreparationMachine{ .diff = machine };

    var state = PreparationState.open_repo;
    while (state != .done) {
        if (machine.cancellable) |cancellable| if (c_libs.g_cancellable_is_cancelled(cancellable) != 0) return preparation_machine.stateFailed(DiffError.Cancelled);

        state = switch (state) {
            .open_repo => try stateOpenRepo(&preparation_machine),
            .checkout_database => try stateCheckoutDatabase(&preparation_machine),
            .load_package_file_map => try stateLoadPackageFileMap(&preparation_machine),
            .cleanup_database => try stateCleanupDatabase(&preparation_machine),
            .close_repo => stateCloseRepo(&preparation_machine),
            .done => unreachable,
        };
    }
}

// ── States ────────────────────────────────────────────────────────────────────
fn stateOpenRepo(machine: *PreparationMachine) DiffError!PreparationState {
    const gfile = c_libs.g_file_new_for_path(machine.diff.data.repo_path);
    defer c_libs.g_object_unref(gfile);

    const repo = c_libs.ostree_repo_new(gfile);
    machine.diff.check(c_libs.ostree_repo_open(repo, machine.diff.cancellable, &machine.diff.gerror), error.RepoOpenFailed) catch |err| {
        c_libs.g_object_unref(repo);
        return machine.stateFailed(err);
    };
    machine.repo = repo;

    return .checkout_database;
}

fn stateCheckoutDatabase(machine: *PreparationMachine) DiffError!PreparationState {
    var checksum: [*c]u8 = null;
    defer if (checksum != null) c_libs.g_free(checksum);

    const timestamp: i64 = @intCast(@divTrunc(std.Io.Clock.real.now(machine.diff.io).nanoseconds, std.time.ns_per_ms));

    const repo = machine.repo orelse return machine.stateFailed(DiffError.RepoOpenFailed);
    const tmp_path = std.mem.span(machine.diff.data.tmp_path);

    machine.diff.check(c_libs.ostree_repo_resolve_rev(repo, machine.currentRef(), 0, &checksum, &machine.diff.gerror), error.CommitNotFound) catch |err| return machine.stateFailed(err);

    const database_temp_dir_name = std.fmt.allocPrint(machine.diff.allocator, "upac-diff-{d}", .{timestamp}) catch return machine.stateFailed(DiffError.AllocFailed);
    defer machine.diff.allocator.free(database_temp_dir_name);

    const database_temp_path = std.fs.path.join(machine.diff.allocator, &.{ tmp_path, database_temp_dir_name }) catch return machine.stateFailed(DiffError.AllocFailed);

    machine.current_database_path = database_temp_path;

    std.Io.Dir.cwd().createDirPath(machine.diff.io, database_temp_path) catch return machine.stateFailed(DiffError.CheckoutFailed);

    checkoutDb(machine, checksum) catch |err| return machine.stateFailed(err);

    return .load_package_file_map;
}

fn stateLoadPackageFileMap(machine: *PreparationMachine) DiffError!PreparationState {
    const database_path = machine.current_database_path orelse return machine.stateFailed(DiffError.CheckoutFailed);

    const database_file_path = std.fs.path.joinZ(machine.diff.allocator, &.{ database_path, DB_NAME }) catch return machine.stateFailed(DiffError.AllocFailed);
    defer machine.diff.allocator.free(database_file_path);

    var base = Database.open(machine.diff.allocator, database_file_path) catch return machine.stateFailed(DiffError.ReadDatabaseFailed);
    defer base.close();

    buildFilePkgMap(base, machine.diff.allocator, &machine.diff.file_pkg_maps[machine.current_ref_index]) catch |err| return machine.stateFailed(err);

    return .cleanup_database;
}

fn stateCleanupDatabase(machine: *PreparationMachine) DiffError!PreparationState {
    if (machine.current_database_path) |path| {
        std.Io.Dir.cwd().deleteTree(machine.diff.io, path) catch {};
        machine.diff.allocator.free(path);
    }
    machine.current_database_path = null;

    machine.current_ref_index += 1;
    if (machine.current_ref_index < 2) return .checkout_database;

    return .close_repo;
}

fn stateCloseRepo(machine: *PreparationMachine) PreparationState {
    if (machine.repo) |repo| {
        c_libs.g_object_unref(repo);
        machine.repo = null;
    }

    return .done;
}
