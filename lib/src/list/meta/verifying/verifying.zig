const std = @import("std");

const types = @import("upac-types");
const PREFIX = types.paths.prefix;
const DB_PATH = types.paths.db_path;
const DB_NAME = types.paths.db_name;

const ListError = types.ListError;

const database = @import("upac-database");
const Database = database.Database;

const list = @import("../meta.zig");
const ListMachine = list.ListMachine;

// ── VerifyingState ────────────────────────────────────────────────────────────
const VerifyingState = enum {
    check_root,
    check_prefix,
    check_db_file,
    open_database,
    close_database,
    done,
};

// ── VerifyingMachine ──────────────────────────────────────────────────────────
const VerifyingMachine = struct {
    list: *ListMachine,
    base: ?Database = null,

    fn stateFailed(self: *VerifyingMachine, err: ListError) ListError {
        if (self.base) |*base| {
            base.close();
            self.base = null;
        }
        return err;
    }
};

// ── Trampoline ────────────────────────────────────────────────────────────────
pub fn run(machine: *ListMachine) ListError!void {
    var verifying_machine = VerifyingMachine{ .list = machine };

    var state = VerifyingState.check_root;

    while (state != .done) {
        state = switch (state) {
            .check_root => try stateCheckRoot(&verifying_machine),
            .check_prefix => try stateCheckPrefix(&verifying_machine),
            .check_db_file => try stateCheckDatabaseFile(&verifying_machine),
            .open_database => try stateOpenDatabase(&verifying_machine),
            .close_database => stateCloseDatabase(&verifying_machine),
            .done => unreachable,
        };
    }
}

// ── States ────────────────────────────────────────────────────────────────────
fn stateCheckRoot(machine: *VerifyingMachine) ListError!VerifyingState {
    const root_path = std.mem.span(machine.list.data.root_path);

    std.Io.Dir.accessAbsolute(machine.list.io, root_path, .{}) catch return ListError.PathNotFound;

    return .check_prefix;
}

fn stateCheckPrefix(machine: *VerifyingMachine) ListError!VerifyingState {
    const root_path = std.mem.span(machine.list.data.root_path);

    const prefix_path = std.fs.path.joinZ(machine.list.allocator, &.{ root_path, PREFIX }) catch return ListError.AllocFailed;
    defer machine.list.allocator.free(prefix_path);

    std.Io.Dir.accessAbsolute(machine.list.io, prefix_path, .{}) catch return ListError.PathNotFound;

    return .check_db_file;
}

fn stateCheckDatabaseFile(machine: *VerifyingMachine) ListError!VerifyingState {
    const root_path = std.mem.span(machine.list.data.root_path);

    const db_file_path = std.fs.path.joinZ(machine.list.allocator, &.{ root_path, PREFIX, DB_PATH, DB_NAME }) catch return ListError.AllocFailed;
    defer machine.list.allocator.free(db_file_path);

    std.Io.Dir.accessAbsolute(machine.list.io, db_file_path, .{}) catch return ListError.DatabaseNotFound;

    return .open_database;
}

fn stateOpenDatabase(machine: *VerifyingMachine) ListError!VerifyingState {
    const root_path = std.mem.span(machine.list.data.root_path);

    const db_file_path = std.fs.path.joinZ(machine.list.allocator, &.{ root_path, PREFIX, DB_PATH, DB_NAME }) catch return machine.stateFailed(ListError.AllocFailed);
    defer machine.list.allocator.free(db_file_path);

    machine.base = Database.open(machine.list.allocator, db_file_path, false) catch |err| return machine.stateFailed(switch (err) {
        error.AccessDenied => ListError.AccessDenied,
        else => ListError.DatabaseNotFound,
    });

    return .close_database;
}

fn stateCloseDatabase(machine: *VerifyingMachine) VerifyingState {
    if (machine.base) |*base| {
        base.close();
        machine.base = null;
    }

    return .done;
}
