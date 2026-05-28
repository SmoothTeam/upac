const std = @import("std");
const c_libs = @import("c-libs");

const types = @import("upac-types");
const PREFIX = types.paths.prefix;
const DB_PATH = types.paths.db_path;
const DB_NAME = types.paths.db_name;

const init_module = @import("../init.zig");
const InitMachine = init_module.InitMachine;
const InitError = init_module.InitError;

// ── VerifyingState ────────────────────────────────────────────────────────────
const VerifyingState = enum {
    check_root,
    check_prefix,
    check_symlink,
    check_repo,
    check_database,
    done,
};

// ── VerifyingMachine ──────────────────────────────────────────────────────────
const VerifyingMachine = struct {
    init: *InitMachine,

    current_symlink_index: usize = 0,
};

// ── Trampoline ────────────────────────────────────────────────────────────────
pub fn run(machine: *InitMachine) InitError!void {
    var verifying_machine = VerifyingMachine{ .init = machine };

    var state = VerifyingState.check_root;
    while (state != .done) {
        if (machine.cancellable) |cancellable| if (c_libs.g_cancellable_is_cancelled(cancellable) != 0) return InitError.Cancelled;

        state = switch (state) {
            .check_root => try stateCheckRoot(&verifying_machine),
            .check_prefix => try stateCheckPrefix(&verifying_machine),
            .check_symlink => try stateCheckSymlink(&verifying_machine),
            .check_repo => try stateCheckRepo(&verifying_machine),
            .check_database => try stateCheckDatabase(&verifying_machine),
            .done => unreachable,
        };
    }
}

// ── States ────────────────────────────────────────────────────────────────────
fn stateCheckRoot(machine: *VerifyingMachine) InitError!VerifyingState {
    const root_path = std.mem.span(machine.init.data.root_path);
    std.Io.Dir.accessAbsolute(machine.init.io, root_path, .{}) catch return InitError.RootNotFound;
    return .check_prefix;
}

fn stateCheckPrefix(machine: *VerifyingMachine) InitError!VerifyingState {
    const root_path = std.mem.span(machine.init.data.root_path);

    const prefix_path = std.fs.path.joinZ(machine.init.allocator, &.{ root_path, PREFIX }) catch return InitError.AllocFailed;
    defer machine.init.allocator.free(prefix_path);

    std.Io.Dir.accessAbsolute(machine.init.io, prefix_path, .{}) catch return .check_symlink;
    return InitError.AlreadyInitialized;
}

fn stateCheckSymlink(machine: *VerifyingMachine) InitError!VerifyingState {
    if (machine.current_symlink_index >= machine.init.data.symlinks.len) return .check_repo;

    const root_path = std.mem.span(machine.init.data.root_path);
    const symlink_name = std.mem.span(machine.init.data.symlinks[machine.current_symlink_index]);

    const link_path = std.fs.path.joinZ(machine.init.allocator, &.{ root_path, symlink_name }) catch return InitError.AllocFailed;
    defer machine.init.allocator.free(link_path);

    var readlink_buf: [std.fs.max_path_bytes]u8 = undefined;
    _ = std.Io.Dir.readLinkAbsolute(machine.init.io, link_path, &readlink_buf) catch {
        machine.current_symlink_index += 1;
        return .check_symlink;
    };

    return InitError.AlreadyInitialized;
}

fn stateCheckRepo(machine: *VerifyingMachine) InitError!VerifyingState {
    const repo_path = std.mem.span(machine.init.data.repo_path);

    const stat = std.Io.Dir.cwd().statFile(machine.init.io, repo_path, .{}) catch return .check_database;

    if (stat.kind == .file) return InitError.NotADirectory;
    if (stat.kind != .directory) return InitError.NotADirectory;

    var repo_dir = std.Io.Dir.openDirAbsolute(machine.init.io, repo_path, .{ .iterate = true }) catch return InitError.RootNotFound;
    defer repo_dir.close(machine.init.io);

    var iter = repo_dir.iterate();
    while (iter.next(machine.init.io) catch return InitError.RootNotFound) |entry| {
        if (std.mem.eql(u8, entry.name, "config")) return InitError.AlreadyInitialized;
    }

    return .check_database;
}

fn stateCheckDatabase(machine: *VerifyingMachine) InitError!VerifyingState {
    const root_path = std.mem.span(machine.init.data.root_path);

    const db_path = std.fs.path.joinZ(machine.init.allocator, &.{ root_path, PREFIX, DB_PATH, DB_NAME }) catch return InitError.AllocFailed;
    defer machine.init.allocator.free(db_path);

    std.Io.Dir.accessAbsolute(machine.init.io, db_path, .{}) catch return .done;
    return InitError.AlreadyInitialized;
}
