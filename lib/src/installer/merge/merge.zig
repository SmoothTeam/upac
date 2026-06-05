const std = @import("std");

const c_libs = @import("c-libs");

const types = @import("upac-types");
const PREFIX = types.paths.prefix;
const CONFIG_DIR = types.paths.config_dir;
const DB_RELATIVE_PATH = types.paths.db_relative_path;

const FileEntry = types.FileEntry;

const database = @import("upac-database");
const Database = database.Database;
const exists = database.packages.exists;
const list = database.files.list;

const installer = @import("../installer.zig");
const InstallerMachine = installer.InstallerMachine;
const InstallerError = installer.InstallerError;

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

    current_package_files: ?[]FileEntry = null,
    current_package_index: usize = 0,

    fn stateFailed(self: *MergeMachine, err: InstallerError) InstallerError {
        if (self.current_package_files) |package_files| {
            for (package_files) |*entry| entry.deinit(self.installer.allocator);
            self.installer.allocator.free(package_files);
            self.current_package_files = null;
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
    while (state != .done) {
        if (machine.cancellable) |cancellable| if (c_libs.g_cancellable_is_cancelled(cancellable) != 0) return merge_machine.stateFailed(InstallerError.Cancelled);

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
    const timestamp: i64 = @intCast(@divTrunc(std.Io.Clock.real.now(machine.installer.io).nanoseconds, std.time.ns_per_ms));
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
    const package_path = std.mem.span(package.temp_package_path);

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

    const temp_database_path = machine.installer.temp_db_path orelse return machine.stateFailed(InstallerError.WriteDatabaseFailed);

    var base = Database.open(machine.installer.allocator, temp_database_path) catch return machine.stateFailed(InstallerError.WriteDatabaseFailed);
    defer base.close();

    const package_uuid = (exists(base, package.meta.name, package.meta.arch, package.meta.arch_sub) catch return machine.stateFailed(InstallerError.PackageNotFound)) orelse return machine.stateFailed(InstallerError.PackageNotFound);

    machine.current_package_files = list(base, package_uuid) catch return machine.stateFailed(InstallerError.WriteDatabaseFailed);

    return .overlay_package_config_dir;
}

fn stateOverlayPackageConfigDir(machine: *MergeMachine) InstallerError!MergeState {
    const package_files = machine.current_package_files orelse return machine.stateFailed(InstallerError.WriteDatabaseFailed);

    const package = machine.installer.data.packages[machine.current_package_index];
    const package_path = std.mem.span(package.temp_package_path);

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
            .file, .sym_link => if (conflict) {
                const checksum: ?[32]u8 = blk: {
                    for (package_files) |file_entry| {
                        if (std.mem.eql(u8, file_entry.path, entry.path)) break :blk file_entry.sha256;
                    }
                    break :blk null;
                };
                resolveConflict(machine, checksum, entry.kind, source_path, destination_path) catch return machine.stateFailed(InstallerError.WriteConfigFailed);
            } else copyEntry(machine, entry.kind, source_path, destination_path) catch return machine.stateFailed(InstallerError.WriteConfigFailed),
            else => {},
        }
    }

    for (package_files) |*file_entry| file_entry.deinit(machine.installer.allocator);
    machine.installer.allocator.free(package_files);
    machine.current_package_files = null;

    machine.current_package_index += 1;
    if (machine.current_package_index < machine.installer.data.packages.len) return .check_package_config_dir;

    return .done;
}
