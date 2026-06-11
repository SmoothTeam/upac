const std = @import("std");

const ffi = @import("upac-ffi");
const CSlice = ffi.CSlice;
const CDiffFileEntry = ffi.CDiffFileEntry;

const types = @import("upac-types");
const PREFIX = types.paths.prefix;
const DB_PATH = types.paths.db_path;
const DB_NAME = types.paths.db_name;

const DiffKind = types.DiffKind;

const database = @import("upac-database");
const Database = database.Database;
const exists = database.packages.exists;
const list_packages = database.packages.list;
const list_files = database.files.list;

const files = @import("../files.zig");
const SearchFilesMachine = files.SearchFilesMachine;
const SearchFilesError = files.SearchFilesError;

const utils = @import("utils.zig");
const containsIgnoreCase = utils.containsIgnoreCase;

// ── SearchState ───────────────────────────────────────────────────────────────
const SearchState = enum {
    open_database,
    fetch,
    close_database,
    done,
};

// ── SearchMachine ─────────────────────────────────────────────────────────────
const SearchMachine = struct {
    searcher: *SearchFilesMachine,

    base: ?Database = null,

    fn stateFailed(self: *SearchMachine, err: SearchFilesError) SearchFilesError {
        if (self.base) |*base| {
            base.close();
            self.base = null;
        }
        return err;
    }
};

// ── Trampoline ────────────────────────────────────────────────────────────────
pub fn run(machine: *SearchFilesMachine) SearchFilesError!void {
    var search_machine = SearchMachine{ .searcher = machine };

    var state = SearchState.open_database;
    while (state != .done) {
        if (machine.data.cancel_token.isCancelled()) return search_machine.stateFailed(SearchFilesError.Cancelled);

        state = switch (state) {
            .open_database => try stateOpenDatabase(&search_machine),
            .fetch => try stateFetch(&search_machine),
            .close_database => stateCloseDatabase(&search_machine),
            .done => unreachable,
        };
    }
}

// ── States ────────────────────────────────────────────────────────────────────
fn stateOpenDatabase(machine: *SearchMachine) SearchFilesError!SearchState {
    const root_path = std.mem.span(machine.searcher.data.root_path);

    const database_file_path = std.fs.path.joinZ(machine.searcher.allocator, &.{ root_path, PREFIX, DB_PATH, DB_NAME }) catch return SearchFilesError.AllocZFailed;
    defer machine.searcher.allocator.free(database_file_path);

    machine.base = Database.open(machine.searcher.allocator, database_file_path, false) catch |err| return machine.stateFailed(switch (err) {
        error.AccessDenied => SearchFilesError.AccessDenied,
        else => SearchFilesError.ReadDatabaseFailed,
    });

    return .fetch;
}

fn stateFetch(machine: *SearchMachine) SearchFilesError!SearchState {
    const base = machine.base orelse return machine.stateFailed(SearchFilesError.ReadDatabaseFailed);
    const query = machine.searcher.data.query;
    const allocator = machine.searcher.allocator;

    const all_packages = list_packages(base) catch return machine.stateFailed(SearchFilesError.ReadDatabaseFailed);
    defer {
        for (all_packages) |*pkg| pkg.deinit(allocator);
        allocator.free(all_packages);
    }

    var matching = std.ArrayList(CDiffFileEntry).empty;
    errdefer {
        for (matching.items) |*entry| entry.free(allocator);
        matching.deinit(allocator);
    }

    for (all_packages) |*pkg| {
        const uuid = exists(base, pkg.name, pkg.arch, pkg.arch_sub) catch continue orelse continue;

        const file_entries = list_files(base, uuid) catch continue;
        defer {
            for (file_entries) |*file_entry| file_entry.deinit(allocator);
            allocator.free(file_entries);
        }

        for (file_entries) |file_entry| {
            if (!containsIgnoreCase(file_entry.path, query)) continue;

            const path_copy = allocator.dupe(u8, file_entry.path) catch return machine.stateFailed(SearchFilesError.OutOfMemory);
            const pkg_name_copy = allocator.dupe(u8, pkg.name) catch {
                allocator.free(path_copy);
                return machine.stateFailed(SearchFilesError.OutOfMemory);
            };

            matching.append(allocator, .{
                .path = CSlice.fromSlice(path_copy),
                .kind = if (file_entry.is_user) DiffKind.modified else DiffKind.added,
                .package_name = CSlice.fromSlice(pkg_name_copy),
                .is_user = file_entry.is_user,
            }) catch {
                allocator.free(path_copy);
                allocator.free(pkg_name_copy);
                return machine.stateFailed(SearchFilesError.OutOfMemory);
            };
        }
    }

    machine.searcher.results = matching.toOwnedSlice(allocator) catch return machine.stateFailed(SearchFilesError.OutOfMemory);

    return .close_database;
}

fn stateCloseDatabase(machine: *SearchMachine) SearchState {
    if (machine.base) |*base| {
        base.close();
        machine.base = null;
    }

    return .done;
}
