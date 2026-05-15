const std = @import("std");

const installer = @import("../installer.zig");
const c_libs = installer.ffi.c_libs;

const database = installer.database;
const FileMap = database.FileMap;
const freeFileMap = database.freeFileMap;
const writePackage = database.writePackage;

const PREFIX = installer.types.PREFIX;
const CONFIG_DIR = installer.types.CONFIG_DIR;
const VAR_DIR = installer.types.VAR_DIR;

const DB_RELATIVE_PATH = installer.types.DB_RELATIVE_PATH;

const InstallerMachine = installer.InstallerMachine;
const InstallerError = installer.InstallerError;

const utils = @import("utils.zig");
const collectChecksums = utils.collectChecksums;
const copyFileTo = utils.copyFileTo;
const copySymlinkTo = utils.copySymlinkTo;

// ── PreparationState ──────────────────────────────────────────────────────────
const PreparationState = enum {
    move_symlinks_to_prefix,
    move_configs_to_prefix,
    create_var_dirs,
    copy_var_files,
    write_databases,
    done,
};

// ── PreparationMachine ────────────────────────────────────────────────────────
const PreparationMachine = struct {
    installer: *InstallerMachine,

    current_packages_index: usize,

    fn stateFailed(self: *PreparationMachine, err: InstallerError) InstallerError {
        _ = self;
        return err;
    }
};

// ── Trampoline ────────────────────────────────────────────────────────────────
pub fn run(machine: *InstallerMachine) InstallerError!void {
    var preparation_machine = PreparationMachine{ .installer = machine, .current_packages_index = 0 };

    var state = PreparationState.move_symlinks_to_prefix;
    if (machine.cancellable) |cancellable| {
        if (c_libs.g_cancellable_is_cancelled(cancellable) != 0) return preparation_machine.stateFailed(InstallerError.Cancelled);
    }

    while (state != .done) {
        state = switch (state) {
            .move_symlinks_to_prefix => try stateMoveSymlinksToPrefix(&preparation_machine),
            .move_configs_to_prefix => try stateMoveConfigsToPrefix(&preparation_machine),
            .create_var_dirs => try stateCreateVarDirs(&preparation_machine),
            .copy_var_files => try stateCopyVarFiles(&preparation_machine),
            .write_databases => try stateWriteDatabases(&preparation_machine),
            .done => unreachable,
        };
    }
}

// ── States ────────────────────────────────────────────────────────────────────
fn stateMoveSymlinksToPrefix(machine: *PreparationMachine) InstallerError!PreparationState {
    const package = machine.installer.data.packages[machine.current_packages_index];
    const package_path = std.mem.span(package.path);

    var package_dir = std.Io.Dir.openDirAbsolute(machine.installer.io, package_path, .{ .iterate = true }) catch return machine.stateFailed(InstallerError.WriteFilesFailed);
    defer package_dir.close(machine.installer.io);

    var package_iter = package_dir.iterate();
    while (package_iter.next(machine.installer.io) catch return machine.stateFailed(InstallerError.WriteFilesFailed)) |dir_entry| {
        if (dir_entry.kind != .directory) continue;
        if (std.mem.eql(u8, dir_entry.name, PREFIX)) continue;
        if (std.mem.eql(u8, dir_entry.name, CONFIG_DIR)) continue;

        const source_path = std.fs.path.joinZ(machine.installer.allocator, &.{ package_path, dir_entry.name }) catch return machine.stateFailed(InstallerError.AllocZFailed);
        defer machine.installer.allocator.free(source_path);

        const destination_path = std.fs.path.joinZ(machine.installer.allocator, &.{ package_path, PREFIX, dir_entry.name }) catch return machine.stateFailed(InstallerError.AllocZFailed);
        defer machine.installer.allocator.free(destination_path);

        const moving_result = std.os.linux.syscall4(
            .renameat,
            @bitCast(@as(isize, std.c.AT.FDCWD)),
            @intFromPtr(source_path.ptr),
            @bitCast(@as(isize, std.c.AT.FDCWD)),
            @intFromPtr(destination_path.ptr),
        );
        if (std.os.linux.errno(moving_result) != .SUCCESS) return machine.stateFailed(InstallerError.WriteFilesFailed);
    }

    return .move_configs_to_prefix;
}

fn stateMoveConfigsToPrefix(machine: *PreparationMachine) InstallerError!PreparationState {
    const package = machine.installer.data.packages[machine.current_packages_index];
    const package_path = std.mem.span(package.path);

    const config_source_path = std.fs.path.joinZ(machine.installer.allocator, &.{ package_path, CONFIG_DIR }) catch return machine.stateFailed(InstallerError.AllocZFailed);
    defer machine.installer.allocator.free(config_source_path);

    const config_destination_path = std.fs.path.joinZ(machine.installer.allocator, &.{ package_path, PREFIX, CONFIG_DIR }) catch return machine.stateFailed(InstallerError.AllocZFailed);
    defer machine.installer.allocator.free(config_destination_path);

    const result = std.os.linux.syscall4(
        .renameat,
        @bitCast(@as(isize, std.c.AT.FDCWD)),
        @intFromPtr(config_source_path.ptr),
        @bitCast(@as(isize, std.c.AT.FDCWD)),
        @intFromPtr(config_destination_path.ptr),
    );
    if (std.os.linux.errno(result) != .SUCCESS) return machine.stateFailed(InstallerError.WriteFilesFailed);

    return .create_var_dirs;
}

fn stateCreateVarDirs(machine: *PreparationMachine) InstallerError!PreparationState {
    const package = machine.installer.data.packages[machine.current_packages_index];
    const package_path = std.mem.span(package.path);

    const var_source_path = std.fs.path.joinZ(machine.installer.allocator, &.{ package_path, VAR_DIR }) catch return machine.stateFailed(InstallerError.AllocZFailed);
    defer machine.installer.allocator.free(var_source_path);

    var var_dir = std.Io.Dir.openDirAbsolute(machine.installer.io, var_source_path, .{ .iterate = true }) catch return machine.stateFailed(InstallerError.WriteFilesFailed);
    defer var_dir.close(machine.installer.io);

    var var_walker = var_dir.walk(machine.installer.allocator) catch return machine.stateFailed(InstallerError.WriteFilesFailed);
    defer var_walker.deinit();

    while (var_walker.next(machine.installer.io) catch return machine.stateFailed(InstallerError.WriteFilesFailed)) |dir_entry| {
        if (dir_entry.kind != .directory) continue;

        const target = std.fs.path.joinZ(machine.installer.allocator, &.{ std.mem.span(machine.installer.data.root_path), VAR_DIR, dir_entry.path }) catch return machine.stateFailed(InstallerError.AllocZFailed);
        defer machine.installer.allocator.free(target);

        std.Io.Dir.cwd().createDirPath(machine.installer.io, target) catch {};
    }

    return .copy_var_files;
}

fn stateCopyVarFiles(machine: *PreparationMachine) InstallerError!PreparationState {
    const package = machine.installer.data.packages[machine.current_packages_index];
    const package_path = std.mem.span(package.path);

    const var_source_path = std.fs.path.join(machine.installer.allocator, &.{ package_path, VAR_DIR }) catch return machine.stateFailed(InstallerError.AllocZFailed);
    defer machine.installer.allocator.free(var_source_path);

    var var_dir = std.Io.Dir.openDirAbsolute(machine.installer.io, var_source_path, .{ .iterate = true }) catch return machine.stateFailed(InstallerError.WriteFilesFailed);
    defer var_dir.close(machine.installer.io);

    var walker = var_dir.walk(machine.installer.allocator) catch return machine.stateFailed(InstallerError.WriteFilesFailed);
    defer walker.deinit();

    while (walker.next(machine.installer.io) catch return machine.stateFailed(InstallerError.WriteFilesFailed)) |ent| {
        if (ent.kind != .file and ent.kind != .sym_link) continue;

        const source_path = std.fs.path.joinZ(machine.installer.allocator, &.{ var_source_path, ent.path }) catch return machine.stateFailed(InstallerError.AllocZFailed);
        defer machine.installer.allocator.free(source_path);

        const destination_path = std.fs.path.joinZ(machine.installer.allocator, &.{ std.mem.span(machine.installer.data.root_path), VAR_DIR, ent.path }) catch return machine.stateFailed(InstallerError.AllocZFailed);
        defer machine.installer.allocator.free(destination_path);

        const exists = blk: {
            std.Io.Dir.accessAbsolute(machine.installer.io, destination_path, .{}) catch break :blk false;
            break :blk true;
        };
        if (exists) continue;

        switch (ent.kind) {
            .file => copyFileTo(machine.installer, source_path, destination_path) catch return machine.stateFailed(InstallerError.WriteFilesFailed),
            .sym_link => copySymlinkTo(machine.installer, source_path, destination_path) catch return machine.stateFailed(InstallerError.WriteFilesFailed),
            else => continue,
        }
    }

    return .write_databases;
}

fn stateWriteDatabases(machine: *PreparationMachine) InstallerError!PreparationState {
    const package = machine.installer.data.packages[machine.current_packages_index];
    const package_temp_path = std.mem.span(package.path);
    const package_checksum = package.checksum;
    const package_meta = package.meta;

    var file_map = FileMap.init(machine.installer.allocator);
    defer freeFileMap(&file_map, machine.installer.allocator);

    const database_dir_path = std.fs.path.join(machine.installer.allocator, &.{ package_temp_path, PREFIX, DB_RELATIVE_PATH }) catch return machine.stateFailed(InstallerError.AllocZFailed);
    defer machine.installer.allocator.free(database_dir_path);

    std.Io.Dir.cwd().createDirPath(machine.installer.io, database_dir_path) catch return machine.stateFailed(InstallerError.WriteDatabaseFailed);

    try collectChecksums(machine.installer, package_temp_path, &file_map);

    writePackage(database_dir_path, package_checksum, package_meta, file_map, machine.installer.allocator) catch return machine.stateFailed(InstallerError.WriteDatabaseFailed);

    machine.current_packages_index += 1;
    if (machine.current_packages_index < machine.installer.data.packages.len) {
        return .move_symlinks_to_prefix;
    }

    return .done;
}
