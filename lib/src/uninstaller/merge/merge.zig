const std = @import("std");

const c_libs = @import("c-libs");

const types = @import("upac-types");
const DB_RELATIVE_PATH = types.DB_RELATIVE_PATH;
const CONFIG_DIR = types.CONFIG_DIR;

const uninstaller = @import("../uninstaller.zig");
const UninstallerMachine = uninstaller.UninstallerMachine;
const UninstallerError = uninstaller.UninstallerError;

const find = @import("upac-index").find;

const database = @import("upac-database");
const readFiles = database.readFiles;
const freeFileMap = database.freeFileMap;

const utils = @import("utils.zig");
const loadParentBody = utils.loadParentBody;
const mirrorDir = utils.mirrorDir;
const computeLiveChecksum = utils.computeLiveChecksum;
const removeEmptyDirs = utils.removeEmptyDirs;

// ── MergeState ────────────────────────────────────────────────────────────────
const MergeState = enum {
    open_repo,
    load_parent_body,
    mirror_config,
    remove_package_configs,
    remove_empty_dirs,
    done,
};

// ── MergeMachine ──────────────────────────────────────────────────────────────
pub const MergeMachine = struct {
    uninstaller: *UninstallerMachine,

    current_package_index: usize = 0,

    repo: ?*c_libs.OstreeRepo = null,

    parent_commit_body: []const u8 = "",

    temp_config_path: ?[:0]u8 = null,

    fn stateFailed(self: *MergeMachine, err: UninstallerError) UninstallerError {
        if (self.repo) |repo| {
            c_libs.g_object_unref(repo);
            self.repo = null;
        }
        if (self.parent_commit_body.len > 0) {
            self.uninstaller.allocator.free(self.parent_commit_body);
            self.parent_commit_body = "";
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
    if (machine.cancellable) |cancellable| if (c_libs.g_cancellable_is_cancelled(cancellable) != 0) return merge_machine.stateFailed(UninstallerError.Cancelled);

    while (state != .done) {
        state = switch (state) {
            .open_repo => try stateOpenRepo(&merge_machine),
            .load_parent_body => try stateLoadParentBody(&merge_machine),
            .mirror_config => try stateMirrorConfig(&merge_machine),
            .remove_package_configs => try stateRemovePackageConfigs(&merge_machine),
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

    return .load_parent_body;
}

fn stateLoadParentBody(machine: *MergeMachine) UninstallerError!MergeState {
    machine.parent_commit_body = loadParentBody(machine) catch |err| return machine.stateFailed(err);

    return .mirror_config;
}

fn stateMirrorConfig(machine: *MergeMachine) UninstallerError!MergeState {
    var timespec: std.os.linux.timespec = undefined;
    var temp_config_dir_name_buf: [128]u8 = undefined;
    _ = std.os.linux.clock_gettime(std.os.linux.CLOCK.REALTIME, &timespec);
    const timestamp: i64 = @as(i64, timespec.sec) * 1000 + @divTrunc(@as(i64, timespec.nsec), 1_000_000);

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
    const package_name = machine.uninstaller.data.package_names[machine.current_package_index];

    const temp_config_path = machine.temp_config_path orelse return machine.stateFailed(UninstallerError.AllocZFailed);

    const root_path = std.mem.span(machine.uninstaller.data.root_path);

    const database_path = std.fs.path.join(machine.uninstaller.allocator, &.{ root_path, DB_RELATIVE_PATH }) catch return machine.stateFailed(UninstallerError.AllocZFailed);
    defer machine.uninstaller.allocator.free(database_path);

    const package_entry = find(machine.parent_commit_body, package_name, machine.uninstaller.allocator) catch return machine.stateFailed(UninstallerError.AllocZFailed);

    const unwrapped_package_entry = package_entry orelse return machine.stateFailed(UninstallerError.PackageNotFound);

    var package_file_map = readFiles(database_path, unwrapped_package_entry.checksum, machine.uninstaller.allocator) catch {
        machine.current_package_index += 1;
        if (machine.current_package_index < machine.uninstaller.data.package_names.len) return .remove_package_configs;
        return .remove_empty_dirs;
    };
    defer freeFileMap(&package_file_map, machine.uninstaller.allocator);

    var package_file_map_iter = package_file_map.iterator();
    while (package_file_map_iter.next()) |file_entry| {
        const package_file_path = file_entry.key_ptr.*;
        const stored_checksum = file_entry.value_ptr.*;

        if (!std.mem.startsWith(u8, package_file_path, CONFIG_DIR ++ "/")) continue;
        const relative = package_file_path[CONFIG_DIR.len + 1 ..];

        const live_path = std.fs.path.joinZ(machine.uninstaller.allocator, &.{ root_path, CONFIG_DIR, relative }) catch continue;
        defer machine.uninstaller.allocator.free(live_path);

        const live_checksum = computeLiveChecksum(machine.uninstaller, live_path) catch continue;
        defer machine.uninstaller.allocator.free(live_checksum);

        if (!std.mem.eql(u8, live_checksum, stored_checksum)) continue;

        const temp_file_path = std.fs.path.joinZ(machine.uninstaller.allocator, &.{ temp_config_path, relative }) catch continue;
        defer machine.uninstaller.allocator.free(temp_file_path);

        std.Io.Dir.deleteFileAbsolute(machine.uninstaller.io, temp_file_path) catch {};
    }

    machine.current_package_index += 1;
    if (machine.current_package_index < machine.uninstaller.data.package_names.len) return .remove_package_configs;

    machine.uninstaller.allocator.free(machine.parent_commit_body);
    machine.parent_commit_body = "";

    return .remove_empty_dirs;
}

fn stateRemoveEmptyDirs(machine: *MergeMachine) UninstallerError!MergeState {
    const temp_config_path = machine.temp_config_path orelse return .done;
    removeEmptyDirs(machine.uninstaller, temp_config_path);
    return .done;
}
