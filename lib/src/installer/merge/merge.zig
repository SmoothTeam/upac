const std = @import("std");

const installer = @import("../installer.zig");
pub const c_libs = installer.ffi.c_libs;

const database = installer.database;
const FileMap = database.FileMap;
const readFiles = database.readFiles;
const freeFileMap = database.freeFileMap;

const PREFIX = installer.types.PREFIX;
const CONFIG_DIR = installer.types.CONFIG_DIR;
const DB_RELATIVE_PATH = installer.types.DB_RELATIVE_PATH;

pub const InstallerMachine = installer.InstallerMachine;
pub const InstallerError = installer.InstallerError;

const utils = @import("utils.zig");
const copyEntry = utils.copyEntry;
const resolveConflict = utils.resolveConflict;

// ── MergeState ────────────────────────────────────────────────────────────────
const MergeState = enum {
    create_temp_config_dir,
    check_package_config_dir,
    load_package_database,
    overlay_package_config_dir,
    done,
};

// ── MergeMachine ──────────────────────────────────────────────────────────────
pub const MergeMachine = struct {
    installer: *InstallerMachine,

    temp_config_path: ?[]u8 = null,

    current_package_database: ?FileMap = null,
    current_package_index: usize = 0,

    fn stateFailed(self: *MergeMachine, err: InstallerError) InstallerError {
        if (self.current_package_database) |*package_database| {
            freeFileMap(package_database, self.installer.allocator);
            self.current_package_database = null;
        }

        if (self.temp_config_path) |path| {
            std.Io.Dir.cwd().deleteTree(self.installer.io, path) catch {};
            self.installer.allocator.free(path);
            self.temp_config_path = null;
        }

        return err;
    }
};

// ── Trampoline ────────────────────────────────────────────────────────────────
pub fn run(machine: *InstallerMachine) InstallerError!void {
    var merge_machine = MergeMachine{ .installer = machine };

    var state = MergeState.create_temp_config_dir;
    if (machine.cancellable) |cancellable| if (c_libs.g_cancellable_is_cancelled(cancellable) != 0) return merge_machine.stateFailed(InstallerError.Cancelled);

    while (state != .done) {
        state = switch (state) {
            .create_temp_config_dir => try stateCreateTempConfigDir(&merge_machine),
            .check_package_config_dir => try stateCheckPackageConfigDir(&merge_machine),
            .load_package_database => try stateLoadPackageDatabase(&merge_machine),
            .overlay_package_config_dir => try stateOverlayPackageConfigDir(&merge_machine),
            .done => unreachable,
        };
    }
}

// ── States ────────────────────────────────────────────────────────────────────
fn stateCreateTempConfigDir(machine: *MergeMachine) InstallerError!MergeState {
    var temp_folder_name_buf: [256]u8 = undefined;
    var timespec: std.os.linux.timespec = undefined;
    _ = std.os.linux.clock_gettime(std.os.linux.CLOCK.REALTIME, &timespec);

    const timestamp: i64 = @as(i64, timespec.sec) * 1000 + @divTrunc(@as(i64, timespec.nsec), 1_000_000);
    const temp_config_folder_name = std.fmt.bufPrintZ(&temp_folder_name_buf, "{s}-install-{d}", .{ CONFIG_DIR, timestamp }) catch return InstallerError.AllocZFailed;

    const temp_config_path = std.fs.path.joinZ(machine.installer.allocator, &.{ std.mem.span(machine.installer.data.root_path), temp_config_folder_name }) catch return machine.stateFailed(InstallerError.AllocZFailed);
    machine.temp_config_path = temp_config_path;
    machine.installer.temp_config_path = temp_config_path.ptr;

    std.Io.Dir.cwd().createDirPath(machine.installer.io, temp_config_path) catch return machine.stateFailed(InstallerError.NotEnoughSpace);

    return .check_package_config_dir;
}

fn stateCheckPackageConfigDir(machine: *MergeMachine) InstallerError!MergeState {
    if (machine.current_package_index >= machine.installer.data.packages.len) return .done;

    const package = machine.installer.data.packages[machine.current_package_index];
    const package_path = std.mem.span(package.path);

    const package_config_path = std.fs.path.joinZ(machine.installer.allocator, &.{ package_path, PREFIX, CONFIG_DIR }) catch return machine.stateFailed(InstallerError.AllocZFailed);
    defer machine.installer.allocator.free(package_config_path);

    std.Io.Dir.accessAbsolute(machine.installer.io, package_config_path, .{}) catch {
        machine.current_package_index += 1;
        return .check_package_config_dir;
    };
    return .load_package_database;
}

fn stateLoadPackageDatabase(machine: *MergeMachine) InstallerError!MergeState {
    const package = machine.installer.data.packages[machine.current_package_index];
    const package_path = std.mem.span(package.path);

    const package_database_path = std.fs.path.join(machine.installer.allocator, &.{ package_path, DB_RELATIVE_PATH }) catch return machine.stateFailed(InstallerError.AllocZFailed);
    defer machine.installer.allocator.free(package_database_path);

    if (machine.current_package_database) |*package_database| {
        freeFileMap(package_database, machine.installer.allocator);
        machine.current_package_database = null;
    }

    machine.current_package_database = readFiles(package_database_path, package.checksum, machine.installer.allocator) catch return machine.stateFailed(InstallerError.WriteDatabaseFailed);

    return .overlay_package_config_dir;
}

fn stateOverlayPackageConfigDir(machine: *MergeMachine) InstallerError!MergeState {
    const package_database = machine.current_package_database orelse return machine.stateFailed(InstallerError.WriteDatabaseFailed);

    const package = machine.installer.data.packages[machine.current_package_index];
    const package_path = std.mem.span(package.path);

    const package_config_path = std.fs.path.join(machine.installer.allocator, &.{ package_path, PREFIX, CONFIG_DIR }) catch return machine.stateFailed(InstallerError.AllocZFailed);
    defer machine.installer.allocator.free(package_config_path);

    var package_config_dir = std.Io.Dir.openDirAbsolute(machine.installer.io, package_config_path, .{ .iterate = true }) catch return machine.stateFailed(InstallerError.WriteConfigFailed);
    defer package_config_dir.close(machine.installer.io);

    var package_condif_dir_walker = package_config_dir.walk(machine.installer.allocator) catch return machine.stateFailed(InstallerError.WriteConfigFailed);
    defer package_condif_dir_walker.deinit();

    while (package_condif_dir_walker.next(machine.installer.io) catch return machine.stateFailed(InstallerError.WriteConfigFailed)) |entry| {
        const source_path = std.fs.path.joinZ(machine.installer.allocator, &.{ package_config_path, entry.path }) catch return machine.stateFailed(InstallerError.AllocZFailed);
        defer machine.installer.allocator.free(source_path);

        const destination_path = std.fs.path.joinZ(machine.installer.allocator, &.{ std.mem.span(machine.installer.data.root_path), CONFIG_DIR, entry.path }) catch return machine.stateFailed(InstallerError.AllocZFailed);
        defer machine.installer.allocator.free(destination_path);

        const conflict = blk: {
            std.Io.Dir.accessAbsolute(machine.installer.io, destination_path, .{}) catch break :blk false;
            break :blk true;
        };

        switch (entry.kind) {
            .directory => std.Io.Dir.cwd().createDirPath(machine.installer.io, destination_path) catch {},
            .file, .sym_link => if (conflict)
                resolveConflict(machine, package_database, entry.kind, source_path, destination_path, entry.path) catch return machine.stateFailed(InstallerError.WriteConfigFailed)
            else
                copyEntry(machine, entry.kind, source_path, destination_path) catch return machine.stateFailed(InstallerError.WriteConfigFailed),
            else => {},
        }
    }

    machine.current_package_index += 1;
    if (machine.current_package_index < machine.installer.data.packages.len) return .check_package_config_dir;

    return .done;
}
