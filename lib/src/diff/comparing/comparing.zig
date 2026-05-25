const std = @import("std");

const types = @import("upac-types");
const FileRecord = types.FileRecord;
const DiffEntry = types.DiffEntry;
const DiffKind = types.DiffKind;

const diff_module = @import("../diff.zig");
const DiffMachine = diff_module.DiffMachine;
const DiffError = diff_module.DiffError;

const utils = @import("utils.zig");
const appendEntry = utils.appendEntry;

// ── ComparingState ────────────────────────────────────────────────────────────
const ComparingState = enum {
    find_removed_and_modified,
    find_added,
    done,
};

// ── ComparingMachine ──────────────────────────────────────────────────────────
const ComparingMachine = struct {
    diff: *DiffMachine,

    entries: std.ArrayList(DiffEntry) = std.ArrayList(DiffEntry).empty,

    fn stateFailed(self: *ComparingMachine, err: DiffError) DiffError {
        for (self.entries.items) |entry| {
            self.diff.allocator.free(entry.path);
            self.diff.allocator.free(entry.package_name);
        }
        self.entries.deinit(self.diff.allocator);
        return err;
    }
};

// ── Trampoline ────────────────────────────────────────────────────────────────
pub fn run(machine: *DiffMachine) DiffError![]DiffEntry {
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
    var iter = machine.diff.file_pkg_maps[0].iterator();
    while (iter.next()) |entry| {
        const path = entry.key_ptr.*;
        const from_record = entry.value_ptr.*;

        if (machine.diff.file_pkg_maps[1].get(path)) |to_record| {
            if (std.mem.eql(u8, &from_record.sha256, &to_record.sha256)) continue;
            appendEntry(&machine.entries, machine.diff.allocator, path, .modified, from_record.pkg_name, from_record.is_user) catch return machine.stateFailed(DiffError.AllocFailed);
        } else {
            appendEntry(&machine.entries, machine.diff.allocator, path, .removed, from_record.pkg_name, from_record.is_user) catch return machine.stateFailed(DiffError.AllocFailed);
        }
    }

    return .find_added;
}

fn stateFindAdded(machine: *ComparingMachine) DiffError!ComparingState {
    var iter = machine.diff.file_pkg_maps[1].iterator();
    while (iter.next()) |entry| {
        const path = entry.key_ptr.*;
        const to_record = entry.value_ptr.*;

        if (machine.diff.file_pkg_maps[0].contains(path)) continue;
        appendEntry(&machine.entries, machine.diff.allocator, path, .added, to_record.pkg_name, to_record.is_user) catch return machine.stateFailed(DiffError.AllocFailed);
    }

    return .done;
}
