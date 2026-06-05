const std = @import("std");

const c_libs = @import("c-libs");

const types = @import("upac-types");
const PREFIX = types.paths.prefix;
const CONFIG_DIR = types.paths.config_dir;
const DB_PATH = types.paths.db_path;
const DB_NAME = types.paths.db_name;

const uninstaller = @import("../uninstaller.zig");
const UninstallerMachine = uninstaller.UninstallerMachine;
const UninstallerError = uninstaller.UninstallerError;

const database = @import("upac-database");
const Database = database.Database;
const exists = database.packages.exists;
const list = database.files.list;

const utils = @import("utils.zig");
const mirrorDir = utils.mirrorDir;
const computeLiveChecksum = utils.computeLiveChecksum;
const removeEmptyDirs = utils.removeEmptyDirs;

// ── MergeState ────────────────────────────────────────────────────────────────
const MergeState = enum {
    open_repo,
    resolve_parent,
    checkout_database,
    close_repo,
    open_database,
    mirror_config,
    remove_package_configs,
    close_database,
    remove_empty_dirs,
    done,
};

// ── MergeMachine ──────────────────────────────────────────────────────────────
pub const MergeMachine = struct {
    uninstaller: *UninstallerMachine,

    current_package_index: usize = 0,

    repo: ?*c_libs.OstreeRepo = null,
    parent_commit_checksum: [65:0]u8 = std.mem.zeroes([65:0]u8),

    base: ?Database = null,
    temp_database_path: ?[]u8 = null,

    temp_config_path: ?[:0]u8 = null,

    fn stateFailed(self: *MergeMachine, err: UninstallerError) UninstallerError {
        if (self.repo) |repo| {
            c_libs.g_object_unref(repo);
            self.repo = null;
        }
        if (self.base) |*base| {
            base.close();
            self.base = null;
        }
        if (self.temp_database_path) |path| {
            std.Io.Dir.cwd().deleteTree(self.uninstaller.io, path) catch {};
            self.uninstaller.allocator.free(path);
            self.temp_database_path = null;
        }
        if (self.temp_config_path) |path| {
            std.Io.Dir.cwd().deleteTree(self.uninstaller.io, path) catch {};
            self.uninstaller.allocator.free(path);
            self.temp_config_path = null;
            self.uninstaller.temp_config_path = null;
        }
        return err;
    }
};

// ── Trampoline ────────────────────────────────────────────────────────────────
pub fn run(machine: *UninstallerMachine) UninstallerError!void {
    var merge_machine = MergeMachine{ .uninstaller = machine };

    var state = MergeState.open_repo;
    while (state != .done) {
        if (machine.cancellable) |cancellable| if (c_libs.g_cancellable_is_cancelled(cancellable) != 0) return merge_machine.stateFailed(UninstallerError.Cancelled);

        state = switch (state) {
            .open_repo => try stateOpenRepo(&merge_machine),
            .resolve_parent => try stateResolveParent(&merge_machine),
            .checkout_database => try stateCheckoutDatabase(&merge_machine),
            .close_repo => stateCloseRepo(&merge_machine),
            .open_database => try stateOpenDatabase(&merge_machine),
            .mirror_config => try stateMirrorConfig(&merge_machine),
            .remove_package_configs => try stateRemovePackageConfigs(&merge_machine),
            .close_database => stateCloseDatabase(&merge_machine),
            .remove_empty_dirs => try stateRemoveEmptyDirs(&merge_machine),
            .done => unreachable,
        };
    }
}

// ── States ────────────────────────────────────────────────────────────────────
fn stateOpenRepo(machine: *MergeMachine) UninstallerError!MergeState {
    const gfile = c_libs.g_file_new_for_path(machine.uninstaller.data.repo_path);
    defer c_libs.g_object_unref(gfile);

    const repo = c_libs.ostree_repo_new(gfile);
    if (c_libs.ostree_repo_open(repo, machine.uninstaller.cancellable, &machine.uninstaller.gerror) == 0) {
        c_libs.g_object_unref(repo);
        return machine.stateFailed(UninstallerError.RepoOpenFailed);
    }
    machine.repo = repo;

    return .resolve_parent;
}

fn stateResolveParent(machine: *MergeMachine) UninstallerError!MergeState {
    const repo = machine.repo orelse return machine.stateFailed(UninstallerError.RepoOpenFailed);

    var head_checksum: [*c]u8 = null;
    defer c_libs.g_free(head_checksum);

    if (c_libs.ostree_repo_resolve_rev(repo, machine.uninstaller.data.branch, 0, &head_checksum, &machine.uninstaller.gerror) == 0) return machine.stateFailed(UninstallerError.CommitNotFound);
    if (head_checksum == null) return machine.stateFailed(UninstallerError.CommitNotFound);

    var head_variant: ?*c_libs.GVariant = null;
    defer if (head_variant) |variant| c_libs.g_variant_unref(variant);

    if (c_libs.ostree_repo_load_variant(repo, c_libs.OSTREE_OBJECT_TYPE_COMMIT, head_checksum, &head_variant, &machine.uninstaller.gerror) == 0) return machine.stateFailed(UninstallerError.CommitNotFound);

    const parent_bytes_variant = c_libs.g_variant_get_child_value(head_variant, 1) orelse return machine.stateFailed(UninstallerError.CommitNotFound);
    defer c_libs.g_variant_unref(parent_bytes_variant);

    var n_bytes: usize = 0;
    const parent_raw = c_libs.g_variant_get_fixed_array(parent_bytes_variant, &n_bytes, 1);
    if (n_bytes != 32) return machine.stateFailed(UninstallerError.CommitNotFound);

    const parent_bytes: *const [32]u8 = @ptrCast(parent_raw);
    @memcpy(machine.parent_commit_checksum[0..64], &std.fmt.bytesToHex(parent_bytes.*, .lower));

    return .checkout_database;
}

fn stateCheckoutDatabase(machine: *MergeMachine) UninstallerError!MergeState {
    const repo = machine.repo orelse return machine.stateFailed(UninstallerError.RepoOpenFailed);
    const root_path = std.mem.span(machine.uninstaller.data.root_path);

    const timestamp: i64 = @intCast(@divTrunc(std.Io.Clock.real.now(machine.uninstaller.io).nanoseconds, std.time.ns_per_ms));

    const temp_dir_name = std.fmt.allocPrint(machine.uninstaller.allocator, "upac-db-uninstall-{d}", .{timestamp}) catch return machine.stateFailed(UninstallerError.AllocZFailed);
    defer machine.uninstaller.allocator.free(temp_dir_name);

    const temp_database_path = std.fs.path.join(machine.uninstaller.allocator, &.{ root_path, temp_dir_name }) catch return machine.stateFailed(UninstallerError.AllocZFailed);
    machine.temp_database_path = temp_database_path;

    std.Io.Dir.cwd().createDirPath(machine.uninstaller.io, temp_database_path) catch return machine.stateFailed(UninstallerError.ReadDatabaseFailed);

    const subpath = std.fs.path.joinZ(machine.uninstaller.allocator, &.{ PREFIX, DB_PATH }) catch return machine.stateFailed(UninstallerError.AllocZFailed);
    defer machine.uninstaller.allocator.free(subpath);

    const temp_database_pathz = machine.uninstaller.allocator.dupeZ(u8, temp_database_path) catch return machine.stateFailed(UninstallerError.AllocZFailed);
    defer machine.uninstaller.allocator.free(temp_database_pathz);

    var checkout_options = std.mem.zeroes(c_libs.OstreeRepoCheckoutAtOptions);
    checkout_options.subpath = subpath;

    if (c_libs.ostree_repo_checkout_at(repo, &checkout_options, c_libs.AT_FDCWD, temp_database_pathz, &machine.parent_commit_checksum, machine.uninstaller.cancellable, &machine.uninstaller.gerror) == 0) return machine.stateFailed(UninstallerError.ReadDatabaseFailed);

    return .close_repo;
}

fn stateCloseRepo(machine: *MergeMachine) MergeState {
    if (machine.repo) |repo| {
        c_libs.g_object_unref(repo);
        machine.repo = null;
    }

    return .open_database;
}

fn stateOpenDatabase(machine: *MergeMachine) UninstallerError!MergeState {
    const temp_database_path = machine.temp_database_path orelse return machine.stateFailed(UninstallerError.ReadDatabaseFailed);

    const database_file_path = std.fs.path.joinZ(machine.uninstaller.allocator, &.{ temp_database_path, DB_NAME }) catch return machine.stateFailed(UninstallerError.AllocZFailed);
    defer machine.uninstaller.allocator.free(database_file_path);

    machine.base = Database.open(machine.uninstaller.allocator, database_file_path) catch return machine.stateFailed(UninstallerError.ReadDatabaseFailed);

    return .mirror_config;
}

fn stateMirrorConfig(machine: *MergeMachine) UninstallerError!MergeState {
    var temp_config_dir_name_buf: [128]u8 = undefined;
    const timestamp: i64 = @intCast(@divTrunc(std.Io.Clock.real.now(machine.uninstaller.io).nanoseconds, std.time.ns_per_ms));

    const root_path = std.mem.span(machine.uninstaller.data.root_path);

    const temp_config_dir_name = std.fmt.bufPrintZ(&temp_config_dir_name_buf, "{s}-uninstall-{d}", .{ CONFIG_DIR, timestamp }) catch return machine.stateFailed(UninstallerError.AllocZFailed);

    const temp_config_path = std.fs.path.joinZ(machine.uninstaller.allocator, &.{ root_path, temp_config_dir_name }) catch |err| return machine.stateFailed(err);

    machine.temp_config_path = temp_config_path;
    machine.uninstaller.temp_config_path = temp_config_path.ptr;

    std.Io.Dir.cwd().createDirPath(machine.uninstaller.io, temp_config_path) catch return machine.stateFailed(UninstallerError.CheckoutFailed);

    const root_config_path = std.fs.path.joinZ(machine.uninstaller.allocator, &.{ root_path, CONFIG_DIR }) catch |err| return machine.stateFailed(err);
    defer machine.uninstaller.allocator.free(root_config_path);

    mirrorDir(machine.uninstaller, root_config_path, temp_config_path) catch |err| return machine.stateFailed(err);

    return .remove_package_configs;
}

fn stateRemovePackageConfigs(machine: *MergeMachine) UninstallerError!MergeState {
    const base = machine.base orelse return machine.stateFailed(UninstallerError.ReadDatabaseFailed);
    const package = machine.uninstaller.data.packages[machine.current_package_index];
    const temp_config_path = machine.temp_config_path orelse return machine.stateFailed(UninstallerError.AllocZFailed);
    const root_path = std.mem.span(machine.uninstaller.data.root_path);

    const uuid = exists(base, package.name, package.arch, package.arch_sub) catch null;

    if (uuid) |package_uuid| {
        const package_files = list(base, package_uuid) catch &.{};
        defer {
            for (package_files) |*file_entry| file_entry.deinit(machine.uninstaller.allocator);
            machine.uninstaller.allocator.free(package_files);
        }

        for (package_files) |file_entry| {
            if (!std.mem.startsWith(u8, file_entry.path, CONFIG_DIR ++ "/")) continue;
            const relative = file_entry.path[CONFIG_DIR.len + 1 ..];

            const live_path = std.fs.path.joinZ(machine.uninstaller.allocator, &.{ root_path, CONFIG_DIR, relative }) catch continue;
            defer machine.uninstaller.allocator.free(live_path);

            const live_sha256 = computeLiveChecksum(machine.uninstaller, live_path) catch continue;
            if (!std.mem.eql(u8, &live_sha256, &file_entry.sha256)) continue;

            const temp_file_path = std.fs.path.joinZ(machine.uninstaller.allocator, &.{ temp_config_path, relative }) catch continue;
            defer machine.uninstaller.allocator.free(temp_file_path);

            std.Io.Dir.deleteFileAbsolute(machine.uninstaller.io, temp_file_path) catch {};
        }
    }

    machine.current_package_index += 1;
    if (machine.current_package_index < machine.uninstaller.data.packages.len) return .remove_package_configs;

    return .close_database;
}

fn stateCloseDatabase(machine: *MergeMachine) MergeState {
    if (machine.base) |*base| {
        base.close();
        machine.base = null;
    }
    if (machine.temp_database_path) |path| {
        std.Io.Dir.cwd().deleteTree(machine.uninstaller.io, path) catch {};
        machine.uninstaller.allocator.free(path);
        machine.temp_database_path = null;
    }

    return .remove_empty_dirs;
}

fn stateRemoveEmptyDirs(machine: *MergeMachine) UninstallerError!MergeState {
    const temp_config_path = machine.temp_config_path orelse return .done;
    removeEmptyDirs(machine.uninstaller, temp_config_path);
    return .done;
}
