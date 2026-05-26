const std = @import("std");

const c_libs = @import("c-libs");

const types = @import("upac-types");
const PREFIX = types.paths.prefix;
const CONFIG_DIR = types.paths.config_dir;
const DB_PATH = types.paths.db_path;
const DB_NAME = types.paths.db_name;

const database = @import("upac-database");
const Database = database.Database;
const packages_list = database.packages.list;
const packages_exists = database.packages.exists;
const files_list = database.files.list;

const rollback = @import("../rollback.zig");
const RollbackMachine = rollback.RollbackMachine;
const RollbackError = rollback.RollbackError;

const utils = @import("utils.zig");
const computeLiveChecksum = utils.computeLiveChecksum;
const removeEmptyDirs = utils.removeEmptyDirs;
const mirrorDir = utils.mirrorDir;
const copyFileTo = utils.copyFileTo;
const copySymlinkTo = utils.copySymlinkTo;

// ── MergeState ────────────────────────────────────────────────────────────────
const MergeState = enum {
    create_temp_config,
    open_database,
    build_file_map,
    close_database,
    overlay_rollback_configs,
    remove_stale_configs,
    remove_empty_dirs,
    done,
};

// ── MergeMachine ──────────────────────────────────────────────────────────────
pub const MergeMachine = struct {
    rollback: *RollbackMachine,

    temp_config_path: ?[:0]u8 = null,

    base: ?Database = null,
    file_map: ?std.StringHashMap([32]u8) = null,

    fn stateFailed(self: *MergeMachine, err: RollbackError) RollbackError {
        if (self.base) |*base| {
            base.close();
            self.base = null;
        }
        if (self.file_map) |*file_map| {
            var iter = file_map.iterator();
            while (iter.next()) |entry| self.rollback.allocator.free(entry.key_ptr.*);
            file_map.deinit();
            self.file_map = null;
        }
        if (self.temp_config_path) |path| {
            std.Io.Dir.cwd().deleteTree(self.rollback.io, path) catch {};
            self.rollback.allocator.free(path);
            self.temp_config_path = null;
            self.rollback.temp_config_path = null;
        }
        return err;
    }
};

// ── Trampoline ────────────────────────────────────────────────────────────────
pub fn run(machine: *RollbackMachine) RollbackError!void {
    var merge_machine = MergeMachine{ .rollback = machine };

    var state = MergeState.create_temp_config;
    if (machine.cancellable) |cancellable| if (c_libs.g_cancellable_is_cancelled(cancellable) != 0) return merge_machine.stateFailed(RollbackError.Cancelled);

    while (state != .done) {
        state = switch (state) {
            .create_temp_config => try stateCreateTempConfigDir(&merge_machine),
            .open_database => try stateOpenDatabase(&merge_machine),
            .build_file_map => try stateBuildFileMap(&merge_machine),
            .close_database => stateCloseDatabase(&merge_machine),
            .overlay_rollback_configs => try stateOverlayRollbackConfigs(&merge_machine),
            .remove_stale_configs => try stateRemoveStaleConfigs(&merge_machine),
            .remove_empty_dirs => stateRemoveEmptyDirs(&merge_machine),
            .done => unreachable,
        };
    }
}

// ── States ────────────────────────────────────────────────────────────────────
fn stateCreateTempConfigDir(machine: *MergeMachine) RollbackError!MergeState {
    var temp_dir_name_buf: [128]u8 = undefined;
    var timespec: std.os.linux.timespec = undefined;
    _ = std.os.linux.clock_gettime(std.os.linux.CLOCK.REALTIME, &timespec);
    const timestamp: i64 = @as(i64, timespec.sec) * 1000 + @divTrunc(@as(i64, timespec.nsec), 1_000_000);

    const root_path = std.mem.span(machine.rollback.data.root_path);

    const temp_dir_name = std.fmt.bufPrintZ(&temp_dir_name_buf, "{s}-rollback-{d}", .{ CONFIG_DIR, timestamp }) catch return machine.stateFailed(RollbackError.AllocZFailed);

    const temp_config_path = std.fs.path.joinZ(machine.rollback.allocator, &.{ root_path, temp_dir_name }) catch return machine.stateFailed(RollbackError.AllocZFailed);
    machine.temp_config_path = temp_config_path;
    machine.rollback.temp_config_path = temp_config_path.ptr;

    std.Io.Dir.cwd().createDirPath(machine.rollback.io, temp_config_path) catch return machine.stateFailed(RollbackError.StagingFailed);

    const root_config_path = std.fs.path.joinZ(machine.rollback.allocator, &.{ root_path, CONFIG_DIR }) catch return machine.stateFailed(RollbackError.AllocZFailed);
    defer machine.rollback.allocator.free(root_config_path);

    mirrorDir(machine.rollback, root_config_path, temp_config_path) catch return machine.stateFailed(RollbackError.StagingFailed);

    return .open_database;
}

fn stateOpenDatabase(machine: *MergeMachine) RollbackError!MergeState {
    const root_path = std.mem.span(machine.rollback.data.root_path);

    const database_file_path = std.fs.path.joinZ(machine.rollback.allocator, &.{ root_path, PREFIX, DB_PATH, DB_NAME }) catch return machine.stateFailed(RollbackError.AllocZFailed);
    defer machine.rollback.allocator.free(database_file_path);

    machine.base = Database.open(machine.rollback.allocator, database_file_path) catch return machine.stateFailed(RollbackError.StagingFailed);

    return .build_file_map;
}

fn stateBuildFileMap(machine: *MergeMachine) RollbackError!MergeState {
    const base = machine.base orelse return machine.stateFailed(RollbackError.StagingFailed);

    var file_map = std.StringHashMap([32]u8).init(machine.rollback.allocator);
    errdefer {
        var iter = file_map.iterator();
        while (iter.next()) |entry| machine.rollback.allocator.free(entry.key_ptr.*);
        file_map.deinit();
    }

    const package_metas = packages_list(base) catch return machine.stateFailed(RollbackError.StagingFailed);
    defer {
        for (package_metas) |*meta| meta.deinit(machine.rollback.allocator);
        machine.rollback.allocator.free(package_metas);
    }

    for (package_metas) |meta| {
        const uuid = packages_exists(base, meta.name, meta.arch, meta.arch_sub) catch continue orelse continue;
        const package_files = files_list(base, uuid) catch continue;
        defer {
            for (package_files) |*file_entry| file_entry.deinit(machine.rollback.allocator);
            machine.rollback.allocator.free(package_files);
        }

        for (package_files) |file_entry| {
            if (!std.mem.startsWith(u8, file_entry.path, CONFIG_DIR ++ "/")) continue;

            const key = machine.rollback.allocator.dupe(u8, file_entry.path) catch continue;
            file_map.put(key, file_entry.sha256) catch {
                machine.rollback.allocator.free(key);
                continue;
            };
        }
    }

    machine.file_map = file_map;
    return .close_database;
}

fn stateCloseDatabase(machine: *MergeMachine) MergeState {
    if (machine.base) |*base| {
        base.close();
        machine.base = null;
    }
    return .overlay_rollback_configs;
}

fn stateOverlayRollbackConfigs(machine: *MergeMachine) RollbackError!MergeState {
    const temp_config_path = machine.temp_config_path orelse return machine.stateFailed(RollbackError.StagingFailed);
    const temp_prefix_path = std.mem.span(machine.rollback.temp_prefix_path orelse return machine.stateFailed(RollbackError.StagingFailed));
    const root_path = std.mem.span(machine.rollback.data.root_path);

    const source_etc = std.fs.path.joinZ(machine.rollback.allocator, &.{ temp_prefix_path, CONFIG_DIR }) catch return machine.stateFailed(RollbackError.AllocZFailed);
    defer machine.rollback.allocator.free(source_etc);

    var dir = std.Io.Dir.openDirAbsolute(machine.rollback.io, source_etc, .{ .iterate = true }) catch return .remove_stale_configs;
    defer dir.close(machine.rollback.io);

    var walker = dir.walk(machine.rollback.allocator) catch return machine.stateFailed(RollbackError.StagingFailed);
    defer walker.deinit();

    while (walker.next(machine.rollback.io) catch return machine.stateFailed(RollbackError.StagingFailed)) |entry| {
        const dest_path = std.fs.path.joinZ(machine.rollback.allocator, &.{ temp_config_path, entry.path }) catch continue;
        defer machine.rollback.allocator.free(dest_path);

        if (entry.kind == .directory) {
            std.Io.Dir.cwd().createDirPath(machine.rollback.io, dest_path) catch {};
            continue;
        }

        if (entry.kind != .file and entry.kind != .sym_link) continue;

        const source_path = std.fs.path.joinZ(machine.rollback.allocator, &.{ source_etc, entry.path }) catch continue;
        defer machine.rollback.allocator.free(source_path);

        const live_path = std.fs.path.joinZ(machine.rollback.allocator, &.{ root_path, CONFIG_DIR, entry.path }) catch continue;
        defer machine.rollback.allocator.free(live_path);

        const live_exists = blk: {
            std.Io.Dir.accessAbsolute(machine.rollback.io, live_path, .{}) catch break :blk false;
            break :blk true;
        };

        if (!live_exists) {
            switch (entry.kind) {
                .file => copyFileTo(machine.rollback, source_path, dest_path) catch {},
                .sym_link => copySymlinkTo(machine.rollback, source_path, dest_path) catch {},
                else => {},
            }
            continue;
        }

        const user_modified = blk: {
            if (entry.kind == .sym_link) break :blk true;

            const db_key = std.fs.path.join(machine.rollback.allocator, &.{ CONFIG_DIR, entry.path }) catch break :blk true;
            defer machine.rollback.allocator.free(db_key);

            const file_map = machine.file_map orelse break :blk true;
            const shipped = file_map.get(db_key) orelse break :blk true;

            const live_checksum = computeLiveChecksum(machine.rollback, live_path) catch break :blk true;

            break :blk !std.mem.eql(u8, &live_checksum, &shipped);
        };

        if (!user_modified) {
            switch (entry.kind) {
                .file => copyFileTo(machine.rollback, source_path, dest_path) catch {},
                .sym_link => copySymlinkTo(machine.rollback, source_path, dest_path) catch {},
                else => {},
            }
        } else {
            const dest_new = std.fmt.allocPrintSentinel(machine.rollback.allocator, "{s}.new", .{dest_path}, 0) catch continue;
            defer machine.rollback.allocator.free(dest_new);
            std.Io.Dir.deleteFileAbsolute(machine.rollback.io, dest_new) catch {};
            switch (entry.kind) {
                .file => copyFileTo(machine.rollback, source_path, dest_new) catch {},
                .sym_link => copySymlinkTo(machine.rollback, source_path, dest_new) catch {},
                else => {},
            }
        }
    }

    return .remove_stale_configs;
}

fn stateRemoveStaleConfigs(machine: *MergeMachine) RollbackError!MergeState {
    const temp_config_path = machine.temp_config_path orelse return .remove_empty_dirs;
    const temp_prefix_path = std.mem.span(machine.rollback.temp_prefix_path orelse return .remove_empty_dirs);
    const root_path = std.mem.span(machine.rollback.data.root_path);
    if (machine.file_map == null) return .remove_empty_dirs;

    var dir = std.Io.Dir.openDirAbsolute(machine.rollback.io, temp_config_path, .{ .iterate = true }) catch return .remove_empty_dirs;
    defer dir.close(machine.rollback.io);

    var walker = dir.walk(machine.rollback.allocator) catch return .remove_empty_dirs;
    defer walker.deinit();

    while (walker.next(machine.rollback.io) catch null) |entry| {
        if (entry.kind != .file and entry.kind != .sym_link) continue;

        const rollback_etc_path = std.fs.path.joinZ(machine.rollback.allocator, &.{ temp_prefix_path, CONFIG_DIR, entry.path }) catch continue;
        defer machine.rollback.allocator.free(rollback_etc_path);

        const in_rollback = blk: {
            std.Io.Dir.accessAbsolute(machine.rollback.io, rollback_etc_path, .{}) catch break :blk false;
            break :blk true;
        };

        if (in_rollback) continue;

        const db_key = std.fs.path.join(machine.rollback.allocator, &.{ CONFIG_DIR, entry.path }) catch continue;
        defer machine.rollback.allocator.free(db_key);

        const shipped = machine.file_map.?.get(db_key) orelse continue;

        const live_path = std.fs.path.joinZ(machine.rollback.allocator, &.{ root_path, CONFIG_DIR, entry.path }) catch continue;
        defer machine.rollback.allocator.free(live_path);

        const live_checksum = computeLiveChecksum(machine.rollback, live_path) catch continue;

        if (std.mem.eql(u8, &live_checksum, &shipped)) {
            const file_in_temp = std.fs.path.joinZ(machine.rollback.allocator, &.{ temp_config_path, entry.path }) catch continue;
            defer machine.rollback.allocator.free(file_in_temp);
            std.Io.Dir.deleteFileAbsolute(machine.rollback.io, file_in_temp) catch {};
        }
    }

    if (machine.file_map) |*file_map| {
        var iter = file_map.iterator();
        while (iter.next()) |entry| machine.rollback.allocator.free(entry.key_ptr.*);
        file_map.deinit();
        machine.file_map = null;
    }

    return .remove_empty_dirs;
}

fn stateRemoveEmptyDirs(machine: *MergeMachine) MergeState {
    const temp_config_path = machine.temp_config_path orelse return .done;
    removeEmptyDirs(machine.rollback, temp_config_path);
    return .done;
}
