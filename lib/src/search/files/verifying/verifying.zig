const std = @import("std");

const types = @import("upac-types");
const PREFIX = types.paths.prefix;
const DB_PATH = types.paths.db_path;
const DB_NAME = types.paths.db_name;

const Database = @import("upac-database").Database;

const files = @import("../files.zig");
const SearchFilesMachine = files.SearchFilesMachine;
const SearchFilesError = files.SearchFilesError;

// ── VerifyingState ────────────────────────────────────────────────────────────
const VerifyingState = enum {
    check_root,
    check_prefix,
    check_database,
    open_database,
    close_database,
    done,
};

// ── VerifyingMachine ──────────────────────────────────────────────────────────
pub const VerifyingMachine = struct {
    searcher: *SearchFilesMachine,

    base: ?Database = null,

    fn stateFailed(self: *VerifyingMachine, err: SearchFilesError) SearchFilesError {
        if (self.base) |*base| {
            base.close();
            self.base = null;
        }
        return err;
    }
};

// ── Trampoline ────────────────────────────────────────────────────────────────
pub fn run(machine: *SearchFilesMachine) SearchFilesError!void {
    var verifying_machine = VerifyingMachine{ .searcher = machine };

    var state = VerifyingState.check_root;
    while (state != .done) {
        if (machine.data.cancel_token.isCancelled()) return verifying_machine.stateFailed(SearchFilesError.Cancelled);
        state = switch (state) {
            .check_root => try stateCheckRoot(&verifying_machine),
            .check_prefix => try stateCheckPrefix(&verifying_machine),
            .check_database => try stateCheckDatabase(&verifying_machine),
            .open_database => try stateOpenDatabase(&verifying_machine),
            .close_database => stateCloseDatabase(&verifying_machine),
            .done => unreachable,
        };
    }
}

// ── States ────────────────────────────────────────────────────────────────────
fn stateCheckRoot(machine: *VerifyingMachine) SearchFilesError!VerifyingState {
    const root_path = std.mem.span(machine.searcher.data.root_path);

    std.Io.Dir.accessAbsolute(machine.searcher.io, root_path, .{}) catch return SearchFilesError.PathNotFound;

    return .check_prefix;
}

fn stateCheckPrefix(machine: *VerifyingMachine) SearchFilesError!VerifyingState {
    const root_path = std.mem.span(machine.searcher.data.root_path);

    const prefix_path = std.fs.path.joinZ(machine.searcher.allocator, &.{ root_path, PREFIX }) catch return SearchFilesError.AllocZFailed;
    defer machine.searcher.allocator.free(prefix_path);

    std.Io.Dir.accessAbsolute(machine.searcher.io, prefix_path, .{}) catch return SearchFilesError.PathNotFound;

    return .check_database;
}

fn stateCheckDatabase(machine: *VerifyingMachine) SearchFilesError!VerifyingState {
    const root_path = std.mem.span(machine.searcher.data.root_path);

    const database_file_path = std.fs.path.joinZ(machine.searcher.allocator, &.{ root_path, PREFIX, DB_PATH, DB_NAME }) catch return SearchFilesError.AllocZFailed;
    defer machine.searcher.allocator.free(database_file_path);

    std.Io.Dir.accessAbsolute(machine.searcher.io, database_file_path, .{}) catch return SearchFilesError.PathNotFound;

    return .open_database;
}

fn stateOpenDatabase(machine: *VerifyingMachine) SearchFilesError!VerifyingState {
    const root_path = std.mem.span(machine.searcher.data.root_path);

    const database_file_path = std.fs.path.joinZ(machine.searcher.allocator, &.{ root_path, PREFIX, DB_PATH, DB_NAME }) catch return SearchFilesError.AllocZFailed;
    defer machine.searcher.allocator.free(database_file_path);

    machine.base = Database.open(machine.searcher.allocator, database_file_path, false) catch |err| return machine.stateFailed(switch (err) {
        error.AccessDenied => SearchFilesError.AccessDenied,
        else => SearchFilesError.ReadDatabaseFailed,
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
