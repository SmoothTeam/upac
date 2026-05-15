// ── Imports ──────────────────────────────────────────────────────────
const std = @import("std");

const diff = @import("diff.zig");
const c_libs = diff.ffi.c_libs;

const CSlice = diff.ffi.CSlice;
const CDiffEntry = diff.ffi.CDiffEntry;

const DiffMachine = diff.DiffMachine;
const DiffError = diff.DiffError;

const utils = @import("utils.zig");
const getRefBody = utils.getRefBody;
const parsePackageBody = utils.parsePackageBody;
const freeStringMap = utils.freeStringMap;

pub fn stateOpenRepo(machine: *DiffMachine) DiffError!void {
    const gfile = c_libs.g_file_new_for_path(machine.data.repo_path);
    defer c_libs.g_object_unref(gfile);

    const repo = c_libs.ostree_repo_new(gfile);
    if (c_libs.ostree_repo_open(repo, machine.cancellable, &machine.gerror) == 0) {
        c_libs.g_object_unref(repo);
        return stateFailed(DiffError.RepoOpenFailed);
    }
    machine.repo = repo;
}

pub fn stateDiffAttributed(machine: *DiffMachine) DiffError![]CDiffEntry {
    var raw_diff_entry_list_c = std.ArrayList(utils.RawDiffEntry).empty;
    defer {
        for (raw_diff_entry_list_c.items) |raw_entry| machine.allocator.free(raw_entry.path);
        raw_diff_entry_list_c.deinit(machine.allocator);
    }

    var diff_entry_list_c = std.ArrayList(CDiffEntry).empty;
    errdefer {
        for (diff_entry_list_c.items) |result_entry| {
            machine.allocator.free(result_entry.path.ptr[0 .. result_entry.path.len + 1]);
            machine.allocator.free(result_entry.package_name.ptr[0 .. result_entry.package_name.len + 1]);
        }
        diff_entry_list_c.deinit(machine.allocator);
    }

    const modified = c_libs.g_ptr_array_new();
    defer c_libs.g_ptr_array_unref(modified);

    const removed = c_libs.g_ptr_array_new();
    defer c_libs.g_ptr_array_unref(removed);

    const added = c_libs.g_ptr_array_new();
    defer c_libs.g_ptr_array_unref(added);

    const from_commit_root = utils.resolveCommitRoot(machine, machine.data.from_ref) catch |err| return err;
    defer c_libs.g_object_unref(from_commit_root);

    if (machine.cancellable) |cancellable| if (c_libs.g_cancellable_is_cancelled(cancellable) != 0) return DiffError.Cancelled;

    const to_commit_root = utils.resolveCommitRoot(machine, machine.data.to_ref) catch |err| return err;
    defer c_libs.g_object_unref(to_commit_root);

    if (c_libs.ostree_diff_dirs(c_libs.OSTREE_DIFF_FLAGS_NONE, from_commit_root, to_commit_root, modified, removed, added, machine.cancellable, &machine.gerror) == 0) {
        if (machine.gerror) |err| if (err.domain == c_libs.g_io_error_quark() and err.code == c_libs.G_IO_ERROR_CANCELLED) return DiffError.Cancelled;
        return DiffError.DiffFailed;
    }

    var file_package_map = std.StringHashMap([]const u8).init(machine.allocator);
    defer utils.freeStringMap(&file_package_map, machine.allocator);

    utils.buildFilePkgMap(machine, machine.data.to_ref, &file_package_map) catch |err| return err;
    utils.buildFilePkgMap(machine, machine.data.from_ref, &file_package_map) catch |err| return err;

    utils.collectEntries(added, to_commit_root, .added, false, &raw_diff_entry_list_c, machine.allocator) catch |err| return err;
    utils.collectEntries(removed, from_commit_root, .removed, false, &raw_diff_entry_list_c, machine.allocator) catch |err| return err;
    utils.collectEntries(modified, to_commit_root, .modified, true, &raw_diff_entry_list_c, machine.allocator) catch |err| return err;

    for (raw_diff_entry_list_c.items) |raw_diff_entry| {
        const package_name = file_package_map.get(raw_diff_entry.path) orelse "";

        const package_name_dupe = machine.allocator.dupeZ(u8, package_name) catch |err| return err;

        const package_path_dupe = machine.allocator.dupeZ(u8, raw_diff_entry.path) catch |err| return err;

        diff_entry_list_c.append(machine.allocator, .{
            .path = CSlice.fromSlice(package_path_dupe),
            .kind = @enumFromInt(@intFromEnum(raw_diff_entry.kind)),
            .package_name = CSlice.fromSlice(package_name_dupe),
        }) catch |err| return err;
    }

    return diff_entry_list_c.toOwnedSlice(machine.allocator) catch |err| return err;
}

pub fn stateFailed(err: DiffError) DiffError {
    return err;
}
