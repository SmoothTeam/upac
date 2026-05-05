// ── Imports ─────────────────────────────────────────────────────────────────────
const std = @import("std");

const data = @import("upac-data");

const uninstaller = @import("uninstaller.zig");
const c_libs = uninstaller.c_libs;

const CSlice = uninstaller.CSlice;

const UninstallerMachine = uninstaller.UninstallerMachine;
const UninstallerError = uninstaller.UninstallerError;

const UninstallStateId = uninstaller.ffi.UninstallStateId;

const utils = @import("utils.zig");

const resolveMtree = utils.resolveMtree;

const removeDbFile = utils.removeDbFile;
const removeFromMtree = utils.removeFromMtree;

const buildCommitBody = utils.buildCommitBody;
const buildCommitSubject = utils.buildCommitSubject;

// ── Trampoline ────────────────────────────────────────────────────────────────
pub fn stateStart(machine: *UninstallerMachine) UninstallerError!void {
    var state = UninstallStateId.verifying;
    while (state != .done) {
        try machine.enter(state);
        state = switch (state) {
            .verifying => try stateVerifying(machine),
            .open_repo => try stateOpenRepo(machine),
            .check_installed => try stateCheckInstalled(machine),
            .load_files => try stateLoadFiles(machine),
            .remove_files => try stateRemoveFiles(machine),
            .remove_db_files => try stateRemoveDbFiles(machine),
            .commit => try stateCommit(machine),
            .checkout_staging => try stateCheckoutStaging(machine),
            .atomic_swap => try stateAtomicSwap(machine),
            .cleanup_staging => try stateCleanupStaging(machine),
            .done, .failed => unreachable,
        };
    }
    try machine.enter(.done);
}

// ── States ────────────────────────────────────────────────────────────────────
fn stateVerifying(machine: *UninstallerMachine) UninstallerError!UninstallStateId {
    const io = std.Io.Threaded.global_single_threaded.io();
    try machine.check(std.Io.Dir.accessAbsolute(io, std.mem.span(machine.data.root_path), .{}), UninstallerError.PathNotFound);
    try machine.check(std.Io.Dir.accessAbsolute(io, std.mem.span(machine.data.repo_path), .{}), UninstallerError.RepoOpenFailed);

    const root_prefix_path_c = try machine.check(std.fs.path.joinZ(machine.allocator, &.{ std.mem.span(machine.data.root_path), std.mem.span(machine.data.prefix_path) }), error.AllocZFailed);
    defer machine.allocator.free(root_prefix_path_c);

    try machine.check(std.Io.Dir.accessAbsolute(io, root_prefix_path_c, .{}), UninstallerError.PathNotFound);

    machine.resetRetries();
    return .open_repo;
}

fn stateOpenRepo(machine: *UninstallerMachine) UninstallerError!UninstallStateId {
    if (machine.mtree) |mtree| c_libs.g_object_unref(mtree);

    const gfile = c_libs.g_file_new_for_path(machine.data.repo_path);
    defer c_libs.g_object_unref(@ptrCast(gfile));

    machine.repo = c_libs.ostree_repo_new(gfile);

    const repo = try machine.unwrap(machine.repo, error.RepoOpenFailed);

    if (c_libs.ostree_repo_open(repo, machine.cancellable, &machine.gerror) == 0) {
        c_libs.g_object_unref(machine.repo);
        return machine.retry(.open_repo);
    }

    try machine.gcheck(c_libs.ostree_repo_prepare_transaction(repo, null, machine.cancellable, &machine.gerror), error.RepoTransactionFailed);

    machine.mtree = resolveMtree(machine, repo);

    machine.resetRetries();
    return .check_installed;
}

fn stateCheckInstalled(machine: *UninstallerMachine) UninstallerError!UninstallStateId {
    var body_len: usize = 0;
    var body_variant: ?*c_libs.GVariant = null;
    defer if (body_variant) |variant| c_libs.g_variant_unref(variant);

    const repo = try machine.unwrap(machine.repo, error.RepoOpenFailed);

    var commit_variant: ?*c_libs.GVariant = null;
    defer if (commit_variant) |variant| c_libs.g_variant_unref(variant);

    if (machine.previous_commit_checksum == null) {
        stateFailed(machine);
        return error.PackageNotFound;
    }

    try machine.gcheck(c_libs.ostree_repo_load_variant(repo, c_libs.OSTREE_OBJECT_TYPE_COMMIT, machine.previous_commit_checksum, &commit_variant, &machine.gerror), error.PackageNotFound);

    body_variant = c_libs.g_variant_get_child_value(commit_variant, 4);
    const body_ptr = c_libs.g_variant_get_string(body_variant, &body_len);
    const body = body_ptr[0..body_len];

    var split_lines_iter = std.mem.splitScalar(u8, body, '\n');
    while (split_lines_iter.next()) |line| {
        const trimmed_line = std.mem.trim(u8, line, " \t\r");
        if (trimmed_line.len == 0) continue;

        const separator_index = std.mem.indexOfScalar(u8, trimmed_line, ' ') orelse continue;
        const pkg_name = trimmed_line[0..separator_index];
        const pkg_checksum = std.mem.trim(u8, trimmed_line[separator_index + 1 ..], " \t");

        if (std.ascii.eqlIgnoreCase(pkg_name, machine.data.package_names[machine.current_package_index])) {
            machine.package_checksum = try machine.allocator.dupe(u8, pkg_checksum);
            machine.resetRetries();
            return .load_files;
        }
    }

    stateFailed(machine);
    return error.PackageNotFound;
}

fn stateLoadFiles(machine: *UninstallerMachine) UninstallerError!UninstallStateId {
    const package_checksum = try machine.unwrap(machine.package_checksum, error.PackageNotFound);

    const abs_database_path = try machine.check(std.fs.path.join(machine.allocator, &.{ std.mem.span(machine.data.root_path), std.mem.span(machine.data.prefix_path), "share/upac/db" }), UninstallerError.AllocZFailed);
    defer machine.allocator.free(abs_database_path);

    machine.package_file_map = try machine.check(data.readFiles(abs_database_path, package_checksum, machine.allocator), UninstallerError.FileMapCorrupted);

    machine.resetRetries();
    return .remove_files;
}

fn stateRemoveFiles(machine: *UninstallerMachine) UninstallerError!UninstallStateId {
    const repo = try machine.unwrap(machine.repo, error.RepoOpenFailed);
    const file_map = try machine.unwrap(machine.package_file_map, error.PackageNotFound);
    const mtree = try machine.unwrap(machine.mtree, error.PackageNotFound);

    var iter = file_map.iterator();
    while (iter.next()) |entry| {
        const stored_path = entry.key_ptr.*;
        removeFromMtree(repo, mtree, stored_path, machine.allocator) catch |err| {
            if (err != error.FileNotFound) {
                stateFailed(machine);
                return UninstallerError.FileMapCorrupted;
            }
        };
    }

    machine.resetRetries();
    return .remove_db_files;
}

fn stateRemoveDbFiles(machine: *UninstallerMachine) UninstallerError!UninstallStateId {
    const repo = try machine.unwrap(machine.repo, error.RepoOpenFailed);
    const mtree = try machine.unwrap(machine.mtree, error.PackageNotFound);
    const pkg_checksum = try machine.unwrap(machine.package_checksum, error.PackageNotFound);

    const relative_database_path = try machine.check(std.fs.path.join(machine.allocator, &.{ std.mem.span(machine.data.prefix_path), "share/upac/db" }), UninstallerError.AllocZFailed);
    defer machine.allocator.free(relative_database_path);

    try removeDbFile(machine, repo, mtree, pkg_checksum, relative_database_path, ".meta");
    try removeDbFile(machine, repo, mtree, pkg_checksum, relative_database_path, ".files");

    if (machine.package_file_map) |*file_map| {
        data.freeFileMap(file_map, machine.allocator);
        machine.package_file_map = null;
    }
    if (machine.package_checksum) |checksum| {
        machine.allocator.free(checksum);
        machine.package_checksum = null;
    }

    machine.current_package_index += 1;
    if (machine.current_package_index < machine.data.package_names.len) {
        machine.resetRetries();
        return .check_installed;
    }

    machine.resetRetries();
    return .commit;
}

fn stateCommit(machine: *UninstallerMachine) UninstallerError!UninstallStateId {
    const repo = try machine.unwrap(machine.repo, error.RepoOpenFailed);
    const mtree = try machine.unwrap(machine.mtree, error.PackageNotFound);

    var body_alloc = std.Io.Writer.Allocating.init(machine.allocator);
    defer body_alloc.deinit();
    try buildCommitBody(machine, repo, machine.previous_commit_checksum, &body_alloc.writer);

    const body_c = try machine.check(machine.allocator.dupeZ(u8, body_alloc.written()), UninstallerError.AllocZFailed);
    defer machine.allocator.free(body_c);

    var out_g_file: ?*c_libs.GFile = null;
    defer if (out_g_file) |g_file| c_libs.g_object_unref(@ptrCast(g_file));
    if (c_libs.ostree_repo_write_mtree(repo, mtree, &out_g_file, machine.cancellable, &machine.gerror) == 0) return machine.retry(.commit);

    const subject_c = try buildCommitSubject(machine);
    defer machine.allocator.free(subject_c);

    var commit_checksum: [*c]u8 = null;
    if (c_libs.ostree_repo_write_commit(repo, machine.previous_commit_checksum, subject_c.ptr, body_c.ptr, null, @as(?*c_libs.OstreeRepoFile, @ptrCast(out_g_file)), &commit_checksum, machine.cancellable, &machine.gerror) == 0) return machine.retry(.commit);

    c_libs.ostree_repo_transaction_set_ref(repo, null, machine.data.branch, commit_checksum);

    if (c_libs.ostree_repo_commit_transaction(repo, null, machine.cancellable, &machine.gerror) == 0) return machine.retry(.commit);
    machine.commit_checksum = commit_checksum;

    machine.resetRetries();
    return .checkout_staging;
}

fn stateCheckoutStaging(machine: *UninstallerMachine) UninstallerError!UninstallStateId {
    const repo = try machine.unwrap(machine.repo, error.AllocZFailed);

    var buf: [256]u8 = undefined;
    var ts: std.os.linux.timespec = undefined;
    _ = std.os.linux.clock_gettime(std.os.linux.CLOCK.REALTIME, &ts);
    const timestamp: i64 = @as(i64, ts.sec) * 1000 + @divTrunc(@as(i64, ts.nsec), 1_000_000);

    const temp_folder_name = try machine.check(std.fmt.bufPrintZ(&buf, "{s}-remove-{d}", .{ machine.data.prefix_path, timestamp }), UninstallerError.AllocZFailed);
    machine.staging_path_c = try machine.check(std.fs.path.joinZ(machine.allocator, &.{ std.mem.span(machine.data.root_path), temp_folder_name }), UninstallerError.AllocZFailed);

    var options = std.mem.zeroes(c_libs.OstreeRepoCheckoutAtOptions);
    options.mode = c_libs.OSTREE_REPO_CHECKOUT_MODE_NONE;
    options.overwrite_mode = c_libs.OSTREE_REPO_CHECKOUT_OVERWRITE_UNION_FILES;

    if (c_libs.ostree_repo_checkout_at(repo, &options, std.c.AT.FDCWD, machine.staging_path_c.?, machine.commit_checksum.?, machine.cancellable, &machine.gerror) == 0) {
        const staging_path_c = try machine.unwrap(machine.staging_path_c, UninstallerError.CheckoutFailed);
        const _io = std.Io.Threaded.global_single_threaded.io();
        try machine.check(std.Io.Dir.cwd().deleteTree(_io, staging_path_c), error.MaxRetriesExceeded);

        machine.staging_path_c = null;

        stateFailed(machine);
        return error.CheckoutFailed;
    }

    return .atomic_swap;
}

fn stateAtomicSwap(machine: *UninstallerMachine) UninstallerError!UninstallStateId {
    const staging_path_c = try machine.unwrap(machine.staging_path_c, error.AllocZFailed);

    const root_prefix_path_c = try machine.check(std.fs.path.joinZ(machine.allocator, &.{ std.mem.span(machine.data.root_path), std.mem.span(machine.data.prefix_path) }), UninstallerError.AllocZFailed);
    defer machine.allocator.free(root_prefix_path_c);

    const staging_prefix_path_c = try machine.check(std.fs.path.joinZ(machine.allocator, &.{ staging_path_c, std.mem.span(machine.data.prefix_path) }), UninstallerError.AllocZFailed);
    defer machine.allocator.free(staging_prefix_path_c);

    const result = std.os.linux.syscall5(.renameat2, @bitCast(@as(isize, std.os.linux.AT.FDCWD)), @intFromPtr(staging_prefix_path_c.ptr), @bitCast(@as(isize, std.os.linux.AT.FDCWD)), @intFromPtr(root_prefix_path_c.ptr), 2);

    if (std.os.linux.errno(result) != .SUCCESS) {
        const io = std.Io.Threaded.global_single_threaded.io();
        try machine.check(std.Io.Dir.cwd().deleteTree(io, staging_path_c), UninstallerError.CheckoutFailed);
    }

    return .cleanup_staging;
}

fn stateCleanupStaging(machine: *UninstallerMachine) UninstallerError!UninstallStateId {
    const staging_path_c = try machine.unwrap(machine.staging_path_c, UninstallerError.AllocZFailed);
    const io = std.Io.Threaded.global_single_threaded.io();
    try machine.check(std.Io.Dir.cwd().deleteTree(io, staging_path_c), UninstallerError.CheckoutFailed);

    return .done;
}

pub fn stateFailed(machine: *UninstallerMachine) void {
    if (machine.stack.items.len > 0 and machine.stack.getLast() == .failed) return;
    if (machine.staging_path_c) |staging_path| {
        const io = std.Io.Threaded.global_single_threaded.io();
        std.Io.Dir.cwd().deleteTree(io, staging_path) catch {};
        machine.allocator.free(staging_path);
        machine.staging_path_c = null;
    }

    if (machine.repo) |repo| {
        _ = c_libs.ostree_repo_abort_transaction(repo, null, &machine.gerror);

        if (machine.commit_checksum != null and machine.previous_commit_checksum != null) {
            _ = c_libs.ostree_repo_set_ref_immediate(repo, null, machine.data.branch, machine.previous_commit_checksum, null, null);
        }
    }

    machine.stack.append(machine.allocator, .failed) catch {};
    machine.report(.failed);
}
