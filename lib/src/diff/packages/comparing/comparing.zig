const std = @import("std");

const types = @import("upac-types");
const DiffKind = types.DiffKind;
const DiffError = types.DiffError;

const packages_module = @import("../packages.zig");
const DiffMachine = packages_module.DiffMachine;
const PackageDiffEntry = packages_module.PackageDiffEntry;

const utils = @import("utils.zig");

// ── ComparingState ────────────────────────────────────────────────────────────
const ComparingState = enum {
    find_removed_and_modified,
    find_added,
    done,
};

// ── ComparingMachine ──────────────────────────────────────────────────────────
const ComparingMachine = struct {
    diff: *DiffMachine,

    entries: std.ArrayList(PackageDiffEntry) = std.ArrayList(PackageDiffEntry).empty,

    fn stateFailed(self: *ComparingMachine, err: DiffError) DiffError {
        for (self.entries.items) |entry| {
            self.diff.allocator.free(entry.name);
            entry.version.deinit(self.diff.allocator);
        }
        self.entries.deinit(self.diff.allocator);
        return err;
    }
};

// ── Trampoline ────────────────────────────────────────────────────────────────
pub fn run(machine: *DiffMachine) DiffError![]PackageDiffEntry {
    var comparing_machine = ComparingMachine{ .diff = machine };

    var state = ComparingState.find_removed_and_modified;
    while (state != .done) {
        state = switch (state) {
            .find_removed_and_modified => try stateFindRemovedAndModified(&comparing_machine),
            .find_added => try stateFindAdded(&comparing_machine),
            .done => unreachable,
        };
    }

    return comparing_machine.entries.toOwnedSlice(machine.allocator);
}

// ── States ────────────────────────────────────────────────────────────────────
fn stateFindRemovedAndModified(machine: *ComparingMachine) DiffError!ComparingState {
    const from_list = machine.diff.packages_lists[0].items;
    const to_list = machine.diff.packages_lists[1].items;

    for (from_list) |from_meta| {
        if (utils.findInList(to_list, from_meta)) |to_meta| {
            if (utils.versionEql(from_meta.version, to_meta.version)) continue;
            utils.appendEntry(&machine.entries, machine.diff.allocator, to_meta, .modified) catch return machine.stateFailed(DiffError.AllocFailed);
        } else {
            utils.appendEntry(&machine.entries, machine.diff.allocator, from_meta, .removed) catch return machine.stateFailed(DiffError.AllocFailed);
        }
    }

    return .find_added;
}

fn stateFindAdded(machine: *ComparingMachine) DiffError!ComparingState {
    const from_list = machine.diff.packages_lists[0].items;
    const to_list = machine.diff.packages_lists[1].items;

    for (to_list) |to_meta| {
        if (utils.findInList(from_list, to_meta) != null) continue;
        utils.appendEntry(&machine.entries, machine.diff.allocator, to_meta, .added) catch return machine.stateFailed(DiffError.AllocFailed);
    }

    return .done;
}
