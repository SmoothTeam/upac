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

const update = @import("../update.zig");
const UpdateMachine = update.UpdateMachine;
const UpdateError = update.UpdateError;

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
    updater: *UpdateMachine,

    temp_config_path: ?[]u8 = null,

    current_package_files: ?[]FileEntry = null,
    current_package_index: usize = 0,

    fn stateFailed(self: *MergeMachine, err: UpdateError) UpdateError {
        if (self.current_package_files) |package_files| {
            for (package_files) |*entry| entry.deinit(self.updater.allocator);
            self.updater.allocator.free(package_files);
            self.current_package_files = null;
        }

        if (self.temp_config_path) |path| {
            std.Io.Dir.cwd().deleteTree(self.updater.io, path) catch {};
            self.updater.allocator.free(path);
            self.temp_config_path = null;
        }

        return err;
    }
};

// ── Trampoline ────────────────────────────────────────────────────────────────
pub fn run(machine: *UpdateMachine) UpdateError!void {
    var merge_machine = MergeMachine{ .updater = machine };

    var state = MergeState.create_temp_config_dir;
    while (state != .done) {
        if (machine.cancellable) |cancellable| if (c_libs.g_cancellable_is_cancelled(cancellable) != 0) return merge_machine.stateFailed(UpdateError.Cancelled);

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
fn stateCreateTempConfigDir(machine: *MergeMachine) UpdateError!MergeState {
    var temp_folder_name_buf: [256]u8 = undefined;
    const timestamp: i64 = @intCast(@divTrunc(std.Io.Clock.real.now(machine.updater.io).nanoseconds, std.time.ns_per_ms));
    const temp_config_folder_name = std.fmt.bufPrintZ(&temp_folder_name_buf, "{s}-install-{d}", .{ CONFIG_DIR, timestamp }) catch return UpdateError.AllocZFailed;

    const temp_config_path = std.fs.path.joinZ(machine.updater.allocator, &.{ std.mem.span(machine.updater.data.root_path), temp_config_folder_name }) catch return machine.stateFailed(UpdateError.AllocZFailed);
    machine.temp_config_path = temp_config_path;
    machine.updater.temp_config_path = temp_config_path.ptr;

    std.Io.Dir.cwd().createDirPath(machine.updater.io, temp_config_path) catch return machine.stateFailed(UpdateError.WriteFilesFailed);

    return .check_package_config_dir;
}

fn stateCheckPackageConfigDir(machine: *MergeMachine) UpdateError!MergeState {
    if (machine.current_package_index >= machine.updater.data.packages.len) return .done;

    const package = machine.updater.data.packages[machine.current_package_index];
    const package_path = std.mem.span(package.temp_package_path);

    const package_config_path = std.fs.path.joinZ(machine.updater.allocator, &.{ package_path, PREFIX, CONFIG_DIR }) catch return machine.stateFailed(UpdateError.AllocZFailed);
    defer machine.updater.allocator.free(package_config_path);

    std.Io.Dir.accessAbsolute(machine.updater.io, package_config_path, .{}) catch {
        machine.current_package_index += 1;
        return .check_package_config_dir;
    };
    return .load_package_database;
}

fn stateLoadPackageDatabase(machine: *MergeMachine) UpdateError!MergeState {
    const package = machine.updater.data.packages[machine.current_package_index];

    const temp_database_path = machine.updater.temp_db_path orelse return machine.stateFailed(UpdateError.WriteDatabaseFailed);

    var base = Database.open(machine.updater.allocator, temp_database_path, false) catch return machine.stateFailed(UpdateError.WriteDatabaseFailed);
    defer base.close();

    const package_uuid = (exists(base, package.meta.name, package.meta.arch, package.meta.arch_sub) catch return machine.stateFailed(UpdateError.PackageNotFound)) orelse return machine.stateFailed(UpdateError.PackageNotFound);

    machine.current_package_files = list(base, package_uuid) catch return machine.stateFailed(UpdateError.WriteDatabaseFailed);

    return .overlay_package_config_dir;
}

fn stateOverlayPackageConfigDir(machine: *MergeMachine) UpdateError!MergeState {
    const package_files = machine.current_package_files orelse return machine.stateFailed(UpdateError.WriteDatabaseFailed);

    const package = machine.updater.data.packages[machine.current_package_index];
    const package_path = std.mem.span(package.temp_package_path);

    const package_config_path = std.fs.path.join(machine.updater.allocator, &.{ package_path, PREFIX, CONFIG_DIR }) catch return machine.stateFailed(UpdateError.AllocZFailed);
    defer machine.updater.allocator.free(package_config_path);

    var package_config_dir = std.Io.Dir.openDirAbsolute(machine.updater.io, package_config_path, .{ .iterate = true }) catch return machine.stateFailed(UpdateError.WriteConfigFailed);
    defer package_config_dir.close(machine.updater.io);

    var package_condif_dir_walker = package_config_dir.walk(machine.updater.allocator) catch return machine.stateFailed(UpdateError.WriteConfigFailed);
    defer package_condif_dir_walker.deinit();

    while (package_condif_dir_walker.next(machine.updater.io) catch return machine.stateFailed(UpdateError.WriteConfigFailed)) |entry| {
        const source_path = std.fs.path.joinZ(machine.updater.allocator, &.{ package_config_path, entry.path }) catch return machine.stateFailed(UpdateError.AllocZFailed);
        defer machine.updater.allocator.free(source_path);

        const destination_path = std.fs.path.joinZ(machine.updater.allocator, &.{ std.mem.span(machine.updater.data.root_path), CONFIG_DIR, entry.path }) catch return machine.stateFailed(UpdateError.AllocZFailed);
        defer machine.updater.allocator.free(destination_path);

        const conflict = blk: {
            std.Io.Dir.accessAbsolute(machine.updater.io, destination_path, .{}) catch break :blk false;
            break :blk true;
        };

        switch (entry.kind) {
            .directory => std.Io.Dir.cwd().createDirPath(machine.updater.io, destination_path) catch {},
            .file, .sym_link => if (conflict) {
                const checksum: ?[32]u8 = blk: {
                    for (package_files) |file_entry| {
                        if (std.mem.eql(u8, file_entry.path, entry.path)) break :blk file_entry.sha256;
                    }
                    break :blk null;
                };
                resolveConflict(machine, checksum, entry.kind, source_path, destination_path) catch return machine.stateFailed(UpdateError.WriteConfigFailed);
            } else copyEntry(machine, entry.kind, source_path, destination_path) catch return machine.stateFailed(UpdateError.WriteConfigFailed),
            else => {},
        }
    }

    for (package_files) |*file_entry| file_entry.deinit(machine.updater.allocator);
    machine.updater.allocator.free(package_files);
    machine.current_package_files = null;

    machine.current_package_index += 1;
    if (machine.current_package_index < machine.updater.data.packages.len) return .check_package_config_dir;

    return .done;
}
