const std = @import("std");

const types = @import("upac-types");
const PREFIX = types.paths.prefix;
const DB_PATH = types.paths.db_path;
const DB_NAME = types.paths.db_name;
const PackageMeta = types.PackageMeta;

const ffi = @import("upac-ffi");
const CPackageMeta = ffi.CPackageMeta;

const database = @import("upac-database");
const Database = database.Database;

const list = @import("../meta.zig");
const ListMachine = list.ListMachine;
const ListError = list.ListError;

const utils = @import("utils.zig");
const convertPackageMeta = utils.convertPackageMeta;

// ── FetchState ────────────────────────────────────────────────────────────────
const FetchState = enum {
    open_database,
    get_packages,
    convert_packages,
    close_database,
    done,
};

// ── FetchMachine ──────────────────────────────────────────────────────────────
const FetchMachine = struct {
    list: *ListMachine,

    base: ?Database = null,

    packages: ?[]PackageMeta = null,
    converted_packages: std.ArrayList(CPackageMeta),

    fn stateFailed(self: *FetchMachine, err: ListError) ListError {
        if (self.packages) |package| {
            for (package) |*package_meta| package_meta.deinit(self.list.allocator);
            self.list.allocator.free(package);
            self.packages = null;
        }

        for (self.converted_packages.items) |*package_meta| package_meta.free(self.list.allocator);
        self.converted_packages.deinit(self.list.allocator);

        if (self.base) |*base| {
            base.close();
            self.base = null;
        }

        return err;
    }
};

// ── Trampoline ────────────────────────────────────────────────────────────────
pub fn run(machine: *ListMachine) ListError![]CPackageMeta {
    var fetch_machine = FetchMachine{
        .list = machine,
        .converted_packages = std.ArrayList(CPackageMeta).empty,
    };

    var state = FetchState.open_database;

    while (state != .done) {
        state = switch (state) {
            .open_database => try stateOpenDatabase(&fetch_machine),
            .get_packages => try stateGetPackages(&fetch_machine),
            .convert_packages => try stateConvertPackages(&fetch_machine),
            .close_database => stateCloseDatabase(&fetch_machine),
            .done => unreachable,
        };
    }

    return fetch_machine.converted_packages.toOwnedSlice(machine.allocator) catch return ListError.AllocFailed;
}

// ── States ────────────────────────────────────────────────────────────────────
fn stateOpenDatabase(machine: *FetchMachine) ListError!FetchState {
    const root_path = std.mem.span(machine.list.data.root_path);

    const db_path = std.fs.path.joinZ(machine.list.allocator, &.{ root_path, PREFIX, DB_PATH, DB_NAME }) catch return machine.stateFailed(ListError.AllocFailed);
    defer machine.list.allocator.free(db_path);

    machine.base = Database.open(machine.list.allocator, db_path) catch return machine.stateFailed(ListError.DatabaseNotFound);

    return .get_packages;
}

fn stateGetPackages(machine: *FetchMachine) ListError!FetchState {
    const base = machine.base orelse return machine.stateFailed(ListError.DatabaseNotFound);

    machine.packages = database.packages.list(base) catch return machine.stateFailed(ListError.FetchFailed);

    return .convert_packages;
}

fn stateConvertPackages(machine: *FetchMachine) ListError!FetchState {
    const packages = machine.packages orelse return machine.stateFailed(ListError.FetchFailed);

    for (packages) |package_meta| {
        const converted = convertPackageMeta(package_meta, machine.list.allocator) catch return machine.stateFailed(ListError.AllocFailed);
        machine.converted_packages.append(machine.list.allocator, converted) catch return machine.stateFailed(ListError.AllocFailed);
    }

    for (packages) |*package_meta| package_meta.deinit(machine.list.allocator);
    machine.list.allocator.free(packages);
    machine.packages = null;

    return .close_database;
}

fn stateCloseDatabase(machine: *FetchMachine) FetchState {
    if (machine.base) |*base| {
        base.close();
        machine.base = null;
    }

    return .done;
}
