const std = @import("std");

const ffi = @import("upac-ffi");
const CPackageMeta = ffi.CPackageMeta;
const CVersion = ffi.CVersion;
const CSlice = ffi.CSlice;
const CArray = ffi.CArray;

const types = @import("upac-types");
const PREFIX = types.paths.prefix;
const DB_PATH = types.paths.db_path;
const DB_NAME = types.paths.db_name;

const PackageMeta = types.PackageMeta;

const database = @import("upac-database");
const Database = database.Database;

const meta = @import("../meta.zig");
const SearchMetaMachine = meta.SearchMetaMachine;
const SearchMetaError = meta.SearchMetaError;

const utils = @import("./utils.zig");
const matchesQuery = utils.matchesQuery;
const toCPackageMeta = utils.toCPackageMeta;
// ── SearchingState ────────────────────────────────────────────────────────────
const SearchingState = enum {
    open_database,
    fetch,
    close_database,
    done,
};

// ── SearchingMachine ──────────────────────────────────────────────────────────
const SearchingMachine = struct {
    searcher: *SearchMetaMachine,

    base: ?Database = null,

    fn stateFailed(self: *SearchingMachine, err: SearchMetaError) SearchMetaError {
        if (self.base) |*base| {
            base.close();
            self.base = null;
        }
        return err;
    }
};

// ── Trampoline ────────────────────────────────────────────────────────────────
pub fn run(machine: *SearchMetaMachine) SearchMetaError!void {
    var searching_machine = SearchingMachine{ .searcher = machine };

    var state = SearchingState.open_database;
    while (state != .done) {
        if (machine.data.cancel_token.isCancelled()) return searching_machine.stateFailed(SearchMetaError.Cancelled);
        state = switch (state) {
            .open_database => try stateOpenDatabase(&searching_machine),
            .fetch => try stateFetch(&searching_machine),
            .close_database => stateCloseDatabase(&searching_machine),
            .done => unreachable,
        };
    }
}

// ── States ────────────────────────────────────────────────────────────────────
fn stateOpenDatabase(machine: *SearchingMachine) SearchMetaError!SearchingState {
    const root_path = std.mem.span(machine.searcher.data.root_path);

    const database_file_path = std.fs.path.joinZ(machine.searcher.allocator, &.{ root_path, PREFIX, DB_PATH, DB_NAME }) catch return SearchMetaError.AllocZFailed;
    defer machine.searcher.allocator.free(database_file_path);

    machine.base = Database.open(machine.searcher.allocator, database_file_path) catch return machine.stateFailed(SearchMetaError.ReadDatabaseFailed);

    return .fetch;
}

fn stateFetch(machine: *SearchingMachine) SearchMetaError!SearchingState {
    const base = machine.base orelse return machine.stateFailed(SearchMetaError.ReadDatabaseFailed);
    const query = machine.searcher.data.query;
    const allocator = machine.searcher.allocator;

    const metas = database.packages.list(base) catch return machine.stateFailed(SearchMetaError.ReadDatabaseFailed);

    var matching = std.ArrayList(CPackageMeta).empty;

    for (metas, 0..) |*pkg_meta, index| {
        if (matchesQuery(pkg_meta, query)) {
            matching.append(allocator, toCPackageMeta(pkg_meta.*)) catch {
                pkg_meta.deinit(allocator);

                for (metas[index + 1 ..]) |*remaining| remaining.deinit(allocator);
                allocator.free(metas);

                for (matching.items) |*matched| matched.free(allocator);
                matching.deinit(allocator);

                return machine.stateFailed(SearchMetaError.OutOfMemory);
            };
        } else {
            pkg_meta.deinit(allocator);
        }
    }
    allocator.free(metas);

    machine.searcher.results = matching.toOwnedSlice(allocator) catch {
        for (matching.items) |*matched| matched.free(allocator);
        matching.deinit(allocator);
        return machine.stateFailed(SearchMetaError.OutOfMemory);
    };

    return .close_database;
}

fn stateCloseDatabase(machine: *SearchingMachine) SearchingState {
    if (machine.base) |*base| {
        base.close();
        machine.base = null;
    }

    return .done;
}
