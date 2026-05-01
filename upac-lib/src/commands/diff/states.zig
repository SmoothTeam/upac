// ── Imports ──────────────────────────────────────────────────────────
const diff = @import("diff.zig");
const std = diff.std;
const c_libs = diff.c_libs;
const data = diff.data;

const DiffMachine = diff.DiffMachine;
const DiffError = diff.DiffError;

const CSlice = diff.ffi.CSlice;
const CPackageDiffEntry = diff.ffi.CPackageDiffEntry;
const CAttributedDiffEntry = diff.ffi.CAttributedDiffEntry;
const CDiffKind = diff.ffi.CDiffKind;
const CPackageDiffKind = diff.ffi.CPackageDiffKind;

const utils = @import("utils.zig");
const getRefBody = utils.getRefBody;
const parsePackageBody = utils.parsePackageBody;
const freeStringMap = utils.freeStringMap;

pub fn stateOpenRepo(machine: *DiffMachine) DiffError!void {
    try machine.check(machine.enter(.open_repo), DiffError.AllocFailed);

    const gfile = c_libs.g_file_new_for_path(machine.data.repo_path);
    defer c_libs.g_object_unref(gfile);

    const repo = c_libs.ostree_repo_new(gfile);
    if (c_libs.ostree_repo_open(repo, machine.cancellable, &machine.gerror) == 0) {
        c_libs.g_object_unref(repo);
        stateFailed(machine);
        return DiffError.RepoOpenFailed;
    }
    machine.repo = repo;
}

pub fn stateDiffPackages(machine: *DiffMachine) DiffError!void {
    try machine.check(machine.enter(.diff_packages), DiffError.AllocFailed);

    const from_body = try machine.check(getRefBody(machine, machine.data.from_ref), DiffError.CommitNotFound);
    defer if (from_body) |body| machine.allocator.free(body);

    const to_body = try machine.check(getRefBody(machine, machine.data.to_ref), DiffError.CommitNotFound);
    defer if (to_body) |body| machine.allocator.free(body);

    var map_from = try machine.check(parsePackageBody(from_body orelse "", machine.allocator), DiffError.AllocFailed);
    defer freeStringMap(&map_from, machine.allocator);

    var map_to = try machine.check(parsePackageBody(to_body orelse "", machine.allocator), DiffError.AllocFailed);
    defer freeStringMap(&map_to, machine.allocator);

    var entries = std.ArrayList(CPackageDiffEntry).empty;
    errdefer {
        for (entries.items) |entry| machine.allocator.free(entry.name.toSlice());
        entries.deinit(machine.allocator);
    }

    var to_iter = map_to.iterator();
    while (to_iter.next()) |entry| {
        const kind: CPackageDiffKind = if (map_from.get(entry.key_ptr.*)) |from_cs|
            (if (std.mem.eql(u8, from_cs, entry.value_ptr.*)) continue else .updated)
        else
            .added;
        const name = try machine.check(machine.allocator.dupe(u8, entry.key_ptr.*), DiffError.AllocFailed);
        try machine.check(entries.append(machine.allocator, .{ .name = CSlice.fromSlice(name), .kind = kind }), DiffError.AllocFailed);
    }

    var from_iter = map_from.iterator();
    while (from_iter.next()) |entry| {
        if (!map_to.contains(entry.key_ptr.*)) {
            const name = try machine.check(machine.allocator.dupe(u8, entry.key_ptr.*), DiffError.AllocFailed);
            try machine.check(entries.append(machine.allocator, .{ .name = CSlice.fromSlice(name), .kind = .removed }), DiffError.AllocFailed);
        }
    }

    machine.result_packages = try machine.check(entries.toOwnedSlice(machine.allocator), DiffError.AllocFailed);
    return stateDone(machine);
}

pub fn stateDiffFilesAttributed(machine: *DiffMachine) DiffError!void {
    try machine.check(machine.enter(.diff_files), DiffError.AllocFailed);

    const modified = c_libs.g_ptr_array_new();
    defer c_libs.g_ptr_array_unref(modified);

    const removed = c_libs.g_ptr_array_new();
    defer c_libs.g_ptr_array_unref(removed);

    const added = c_libs.g_ptr_array_new();
    defer c_libs.g_ptr_array_unref(added);

    const from_root = try machine.check(utils.resolveCommitRoot(machine, machine.data.from_ref), DiffError.CommitNotFound);
    defer c_libs.g_object_unref(from_root);

    if (diff.ffi.isCancelRequested()) return DiffError.Cancelled;

    const to_root = try machine.check(utils.resolveCommitRoot(machine, machine.data.to_ref), DiffError.CommitNotFound);
    defer c_libs.g_object_unref(to_root);

    if (c_libs.ostree_diff_dirs(c_libs.OSTREE_DIFF_FLAGS_NONE, from_root, to_root, modified, removed, added, machine.cancellable, &machine.gerror) == 0) {
        if (machine.gerror) |err| if (err.domain == c_libs.g_io_error_quark() and err.code == c_libs.G_IO_ERROR_CANCELLED) return DiffError.Cancelled;
        stateFailed(machine);
        return DiffError.DiffFailed;
    }

    var file_pkg = std.StringHashMap([]const u8).init(machine.allocator);
    defer utils.freeStringMap(&file_pkg, machine.allocator);
    try machine.check(utils.buildFilePkgMap(machine, machine.data.to_ref, &file_pkg), DiffError.DiffFailed);
    try machine.check(utils.buildFilePkgMap(machine, machine.data.from_ref, &file_pkg), DiffError.DiffFailed);

    var raw = std.ArrayList(utils.RawDiffEntry).empty;
    errdefer {
        for (raw.items) |raw_entry| machine.allocator.free(raw_entry.path);
        raw.deinit(machine.allocator);
    }
    try machine.check(utils.collectEntries(added, to_root, .added, false, &raw, machine.allocator), DiffError.AllocFailed);
    try machine.check(utils.collectEntries(removed, from_root, .removed, false, &raw, machine.allocator), DiffError.AllocFailed);
    try machine.check(utils.collectEntries(modified, to_root, .modified, true, &raw, machine.allocator), DiffError.AllocFailed);

    var result = std.ArrayList(CAttributedDiffEntry).empty;
    errdefer {
        for (result.items) |result_entry| {
            machine.allocator.free(result_entry.path.toSlice());
            machine.allocator.free(result_entry.package_name.toSlice());
        }
        result.deinit(machine.allocator);
    }

    for (raw.items) |raw_entry| {
        const package_name = file_pkg.get(raw_entry.path) orelse "";
        const path_dupe = try machine.check(machine.allocator.dupe(u8, raw_entry.path), DiffError.AllocFailed);
        const package_name_dupe = try machine.check(machine.allocator.dupe(u8, package_name), DiffError.AllocFailed);
        try machine.check(result.append(machine.allocator, .{
            .path = CSlice.fromSlice(path_dupe),
            .kind = @enumFromInt(@intFromEnum(raw_entry.kind)),
            .package_name = CSlice.fromSlice(package_name_dupe),
        }), DiffError.AllocFailed);
    }

    machine.result_files = try machine.check(result.toOwnedSlice(machine.allocator), DiffError.AllocFailed);
    return stateDone(machine);
}

fn stateDone(machine: *DiffMachine) DiffError!void {
    try machine.check(machine.enter(.done), DiffError.AllocFailed);
}

pub fn stateFailed(machine: *DiffMachine) void {
    _ = machine.enter(.failed) catch {};
}
