const std = @import("std");

const c_libs = @import("c-libs");

const types = @import("upac-types");
const PREFIX = types.paths.prefix;
const CONFIG_DIR = types.paths.config_dir;
const VAR_DIR = types.paths.var_dir;
const DB_PATH = types.paths.db_path;
const DB_NAME = types.paths.db_name;
const FileEntry = types.FileEntry;

const database = @import("upac-database");
const Database = database.Database;

const packages_exists = database.packages.exists;
const packages_update = database.packages.update;
const files_list = database.files.list;
const files_update = database.files.update;
const files_delete = database.files.delete;

const update = @import("../update.zig");
const UpdateMachine = update.UpdateMachine;
const UpdateError = update.UpdateError;

const utils = @import("utils.zig");
const collectChecksums = utils.collectChecksums;
const copyFileTo = utils.copyFileTo;
const copySymlinkTo = utils.copySymlinkTo;

// ── PreparationState ──────────────────────────────────────────────────────────
const PreparationState = enum {
    create_db_temp,
    copy_current_db,
    move_symlinks_to_prefix,
    move_configs_to_prefix,
    create_var_dirs,
    copy_var_files,
    open_database,
    update_package_meta,
    update_file_records,
    close_database,
    done,
};

// ── PreparationMachine ────────────────────────────────────────────────────────
const PreparationMachine = struct {
    updater: *UpdateMachine,

    current_packages_index: usize,

    base: ?Database = null,
    old_uuid: ?[16]u8 = null,
    deleted_paths: std.ArrayList([]const u8) = std.ArrayList([]const u8).empty,

    fn stateFailed(self: *PreparationMachine, err: UpdateError) UpdateError {
        for (self.deleted_paths.items) |path| self.updater.allocator.free(path);
        self.deleted_paths.deinit(self.updater.allocator);

        if (self.base) |*base| {
            base.close();
            self.base = null;
        }

        if (self.updater.temp_db_path) |path| {
            std.Io.Dir.cwd().deleteTree(self.updater.io, std.mem.span(path)) catch {};
            self.updater.allocator.free(std.mem.span(path));
            self.updater.temp_db_path = null;
        }

        return err;
    }
};

// ── Trampoline ────────────────────────────────────────────────────────────────
pub fn run(machine: *UpdateMachine) UpdateError!void {
    var preparation_machine = PreparationMachine{ .updater = machine, .current_packages_index = 0 };

    var state = PreparationState.create_db_temp;
    while (state != .done) {
        if (machine.cancellable) |cancellable| if (c_libs.g_cancellable_is_cancelled(cancellable) != 0) return preparation_machine.stateFailed(UpdateError.Cancelled);

        state = switch (state) {
            .create_db_temp => try stateCreateDbTemp(&preparation_machine),
            .copy_current_db => try stateCopyCurrentDb(&preparation_machine),
            .move_symlinks_to_prefix => try stateMoveSymlinksToPrefix(&preparation_machine),
            .move_configs_to_prefix => try stateMoveConfigsToPrefix(&preparation_machine),
            .create_var_dirs => try stateCreateVarDirs(&preparation_machine),
            .copy_var_files => try stateCopyVarFiles(&preparation_machine),
            .open_database => try stateOpenDatabase(&preparation_machine),
            .update_package_meta => try stateUpdatePackageMeta(&preparation_machine),
            .update_file_records => try stateUpdateFileRecords(&preparation_machine),
            .close_database => stateCloseDatabase(&preparation_machine),
            .done => unreachable,
        };
    }
}

// ── States ────────────────────────────────────────────────────────────────────
fn stateCreateDbTemp(machine: *PreparationMachine) UpdateError!PreparationState {
    const timestamp: i64 = @intCast(@divTrunc(std.Io.Clock.real.now(machine.updater.io).nanoseconds, std.time.ns_per_ms));

    const tmp_path = std.mem.span(machine.updater.data.tmp_path);

    const temp_database_name = std.fmt.allocPrint(machine.updater.allocator, "upac-db-{d}", .{timestamp}) catch return machine.stateFailed(UpdateError.AllocZFailed);
    errdefer machine.updater.allocator.free(temp_database_name);

    const temp_database_path = std.fs.path.joinZ(machine.updater.allocator, &.{ tmp_path, temp_database_name }) catch return machine.stateFailed(UpdateError.AllocZFailed);
    errdefer machine.updater.allocator.free(temp_database_path);

    machine.updater.temp_db_path = temp_database_path;

    std.Io.Dir.cwd().createDirPath(machine.updater.io, temp_database_path) catch return machine.stateFailed(UpdateError.WriteDatabaseFailed);

    return .copy_current_db;
}

fn stateCopyCurrentDb(machine: *PreparationMachine) UpdateError!PreparationState {
    const root_path = std.mem.span(machine.updater.data.root_path);
    const temp_database_path = machine.updater.temp_db_path orelse return machine.stateFailed(UpdateError.WriteDatabaseFailed);

    const source_database_path = std.fs.path.joinZ(machine.updater.allocator, &.{ root_path, PREFIX, DB_PATH, DB_NAME }) catch return machine.stateFailed(UpdateError.AllocZFailed);
    defer machine.updater.allocator.free(source_database_path);

    std.Io.Dir.copyFileAbsolute(source_database_path, std.mem.span(temp_database_path), machine.updater.io, .{}) catch return UpdateError.WriteDatabaseFailed;

    return .move_symlinks_to_prefix;
}

fn stateMoveSymlinksToPrefix(machine: *PreparationMachine) UpdateError!PreparationState {
    const package = machine.updater.data.packages[machine.current_packages_index];
    const package_path = std.mem.span(package.temp_package_path);

    var package_dir = std.Io.Dir.openDirAbsolute(machine.updater.io, package_path, .{ .iterate = true }) catch return machine.stateFailed(UpdateError.WriteFilesFailed);
    defer package_dir.close(machine.updater.io);

    var package_iter = package_dir.iterate();
    while (package_iter.next(machine.updater.io) catch return machine.stateFailed(UpdateError.WriteFilesFailed)) |dir_entry| {
        if (dir_entry.kind != .directory) continue;
        if (std.mem.eql(u8, dir_entry.name, PREFIX)) continue;
        if (std.mem.eql(u8, dir_entry.name, CONFIG_DIR)) continue;

        const source_path = std.fs.path.joinZ(machine.updater.allocator, &.{ package_path, dir_entry.name }) catch return machine.stateFailed(UpdateError.AllocZFailed);
        defer machine.updater.allocator.free(source_path);

        const destination_path = std.fs.path.joinZ(machine.updater.allocator, &.{ package_path, PREFIX, dir_entry.name }) catch return machine.stateFailed(UpdateError.AllocZFailed);
        defer machine.updater.allocator.free(destination_path);

        const moving_result = std.os.linux.syscall4(
            .renameat,
            @bitCast(@as(isize, std.c.AT.FDCWD)),
            @intFromPtr(source_path.ptr),
            @bitCast(@as(isize, std.c.AT.FDCWD)),
            @intFromPtr(destination_path.ptr),
        );
        if (std.os.linux.errno(moving_result) != .SUCCESS) return machine.stateFailed(UpdateError.WriteFilesFailed);
    }

    return .move_configs_to_prefix;
}

fn stateMoveConfigsToPrefix(machine: *PreparationMachine) UpdateError!PreparationState {
    const package = machine.updater.data.packages[machine.current_packages_index];
    const package_path = std.mem.span(package.temp_package_path);

    const config_source_path = std.fs.path.joinZ(machine.updater.allocator, &.{ package_path, CONFIG_DIR }) catch return machine.stateFailed(UpdateError.AllocZFailed);
    defer machine.updater.allocator.free(config_source_path);

    const config_destination_path = std.fs.path.joinZ(machine.updater.allocator, &.{ package_path, PREFIX, CONFIG_DIR }) catch return machine.stateFailed(UpdateError.AllocZFailed);
    defer machine.updater.allocator.free(config_destination_path);

    const result = std.os.linux.syscall4(
        .renameat,
        @bitCast(@as(isize, std.c.AT.FDCWD)),
        @intFromPtr(config_source_path.ptr),
        @bitCast(@as(isize, std.c.AT.FDCWD)),
        @intFromPtr(config_destination_path.ptr),
    );
    if (std.os.linux.errno(result) != .SUCCESS) return machine.stateFailed(UpdateError.WriteFilesFailed);

    return .create_var_dirs;
}

fn stateCreateVarDirs(machine: *PreparationMachine) UpdateError!PreparationState {
    const package = machine.updater.data.packages[machine.current_packages_index];
    const package_path = std.mem.span(package.temp_package_path);

    const var_source_path = std.fs.path.joinZ(machine.updater.allocator, &.{ package_path, VAR_DIR }) catch return machine.stateFailed(UpdateError.AllocZFailed);
    defer machine.updater.allocator.free(var_source_path);

    var var_dir = std.Io.Dir.openDirAbsolute(machine.updater.io, var_source_path, .{ .iterate = true }) catch return machine.stateFailed(UpdateError.WriteFilesFailed);
    defer var_dir.close(machine.updater.io);

    var var_walker = var_dir.walk(machine.updater.allocator) catch return machine.stateFailed(UpdateError.WriteFilesFailed);
    defer var_walker.deinit();

    while (var_walker.next(machine.updater.io) catch return machine.stateFailed(UpdateError.WriteFilesFailed)) |dir_entry| {
        if (dir_entry.kind != .directory) continue;

        const target = std.fs.path.joinZ(machine.updater.allocator, &.{ std.mem.span(machine.updater.data.root_path), VAR_DIR, dir_entry.path }) catch return machine.stateFailed(UpdateError.AllocZFailed);
        defer machine.updater.allocator.free(target);

        std.Io.Dir.cwd().createDirPath(machine.updater.io, target) catch {};
    }

    return .copy_var_files;
}

fn stateCopyVarFiles(machine: *PreparationMachine) UpdateError!PreparationState {
    const package = machine.updater.data.packages[machine.current_packages_index];
    const package_path = std.mem.span(package.temp_package_path);

    const var_source_path = std.fs.path.join(machine.updater.allocator, &.{ package_path, VAR_DIR }) catch return machine.stateFailed(UpdateError.AllocZFailed);
    defer machine.updater.allocator.free(var_source_path);

    var var_dir = std.Io.Dir.openDirAbsolute(machine.updater.io, var_source_path, .{ .iterate = true }) catch return machine.stateFailed(UpdateError.WriteFilesFailed);
    defer var_dir.close(machine.updater.io);

    var walker = var_dir.walk(machine.updater.allocator) catch return machine.stateFailed(UpdateError.WriteFilesFailed);
    defer walker.deinit();

    while (walker.next(machine.updater.io) catch return machine.stateFailed(UpdateError.WriteFilesFailed)) |ent| {
        if (ent.kind != .file and ent.kind != .sym_link) continue;

        const source_path = std.fs.path.joinZ(machine.updater.allocator, &.{ var_source_path, ent.path }) catch return machine.stateFailed(UpdateError.AllocZFailed);
        defer machine.updater.allocator.free(source_path);

        const destination_path = std.fs.path.joinZ(machine.updater.allocator, &.{ std.mem.span(machine.updater.data.root_path), VAR_DIR, ent.path }) catch return machine.stateFailed(UpdateError.AllocZFailed);
        defer machine.updater.allocator.free(destination_path);

        const already_exists = blk: {
            std.Io.Dir.accessAbsolute(machine.updater.io, destination_path, .{}) catch break :blk false;
            break :blk true;
        };
        if (already_exists) continue;

        switch (ent.kind) {
            .file => copyFileTo(machine.updater, source_path, destination_path) catch return machine.stateFailed(UpdateError.WriteFilesFailed),
            .sym_link => copySymlinkTo(machine.updater, source_path, destination_path) catch return machine.stateFailed(UpdateError.WriteFilesFailed),
            else => continue,
        }
    }

    return .open_database;
}

fn stateOpenDatabase(machine: *PreparationMachine) UpdateError!PreparationState {
    const package = machine.updater.data.packages[machine.current_packages_index];
    const temp_database_path = machine.updater.temp_db_path orelse return machine.stateFailed(UpdateError.WriteDatabaseFailed);

    machine.base = Database.open(machine.updater.allocator, temp_database_path) catch return machine.stateFailed(UpdateError.WriteDatabaseFailed);

    const base = machine.base orelse return machine.stateFailed(UpdateError.WriteDatabaseFailed);
    machine.old_uuid = (packages_exists(base, package.meta.name, package.meta.arch, package.meta.arch_sub) catch return machine.stateFailed(UpdateError.WriteDatabaseFailed)) orelse return machine.stateFailed(UpdateError.WriteDatabaseFailed);

    return .update_package_meta;
}

fn stateUpdatePackageMeta(machine: *PreparationMachine) UpdateError!PreparationState {
    const package = machine.updater.data.packages[machine.current_packages_index];
    const base = machine.base orelse return machine.stateFailed(UpdateError.WriteDatabaseFailed);

    packages_update(base, package.meta) catch return machine.stateFailed(UpdateError.WriteDatabaseFailed);

    return .update_file_records;
}

fn stateUpdateFileRecords(machine: *PreparationMachine) UpdateError!PreparationState {
    const package = machine.updater.data.packages[machine.current_packages_index];
    const package_path = std.mem.span(package.temp_package_path);
    const base = machine.base orelse return machine.stateFailed(UpdateError.WriteDatabaseFailed);
    const old_uuid = machine.old_uuid orelse return machine.stateFailed(UpdateError.WriteDatabaseFailed);

    var new_file_entries = std.ArrayList(FileEntry).empty;
    defer {
        for (new_file_entries.items) |*entry| entry.deinit(machine.updater.allocator);
        new_file_entries.deinit(machine.updater.allocator);
    }

    collectChecksums(machine.updater, package_path, &new_file_entries) catch return machine.stateFailed(UpdateError.WriteDatabaseFailed);

    const old_file_entries = files_list(base, old_uuid) catch return machine.stateFailed(UpdateError.WriteDatabaseFailed);
    defer {
        for (old_file_entries) |*entry| entry.deinit(machine.updater.allocator);
        machine.updater.allocator.free(old_file_entries);
    }

    for (new_file_entries.items) |new_entry| {
        const is_unchanged = blk: {
            for (old_file_entries) |old_entry| {
                if (!std.mem.eql(u8, old_entry.path, new_entry.path)) continue;
                break :blk std.mem.eql(u8, &old_entry.sha256, &new_entry.sha256);
            }
            break :blk false;
        };

        if (is_unchanged) {
            const abs_path = std.fs.path.joinZ(machine.updater.allocator, &.{ package_path, new_entry.path }) catch return machine.stateFailed(UpdateError.AllocZFailed);
            defer machine.updater.allocator.free(abs_path);
            std.Io.Dir.deleteFileAbsolute(machine.updater.io, abs_path) catch {};
        }

        files_update(base, old_uuid, new_entry) catch return machine.stateFailed(UpdateError.WriteDatabaseFailed);
    }

    outer: for (old_file_entries) |old_entry| {
        for (new_file_entries.items) |new_entry| if (std.mem.eql(u8, old_entry.path, new_entry.path)) continue :outer;

        const path_copy = machine.updater.allocator.dupe(u8, old_entry.path) catch return machine.stateFailed(UpdateError.AllocZFailed);
        machine.deleted_paths.append(machine.updater.allocator, path_copy) catch {
            machine.updater.allocator.free(path_copy);
            return machine.stateFailed(UpdateError.AllocZFailed);
        };

        files_delete(base, old_uuid, old_entry.path) catch return machine.stateFailed(UpdateError.WriteDatabaseFailed);
    }

    return .close_database;
}

fn stateCloseDatabase(machine: *PreparationMachine) PreparationState {
    if (machine.base) |*base| {
        base.close();
        machine.base = null;
    }
    machine.old_uuid = null;

    machine.updater.deleted_file_paths = machine.deleted_paths.toOwnedSlice(machine.updater.allocator) catch null;
    machine.deleted_paths = std.ArrayList([]const u8).empty;

    machine.current_packages_index += 1;
    if (machine.current_packages_index < machine.updater.data.packages.len) return .move_symlinks_to_prefix;

    return .done;
}
