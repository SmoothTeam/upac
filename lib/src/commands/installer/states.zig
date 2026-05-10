// ── Imports ─────────────────────────────────────────────────────────────────────
const installer = @import("installer.zig");
const std = installer.std;
const c_libs = installer.c_libs;
const data = installer.data;

const PREFIX = installer.PREFIX;
const DB_RELATIVE_PATH = installer.DB_RELATIVE_PATH;

const InstallerMachine = installer.InstallerMachine;
const InstallerError = installer.InstallerError;

const find = data.find;
const append = data.append;
const remove = data.remove;

const utils = @import("utils.zig");
const dirSize = utils.dirSize;
const collectFileChecksums = utils.collectFileChecksums;
const estimateCheckoutSize = utils.estimateCheckoutSize;

const loadCommitBody = utils.loadCommitBody;
const mergeDirs = utils.mergeDirs;
const mirrorDir = utils.mirrorDir;
const overlayDir = utils.overlayDir;
const copyFileTo = utils.copyFileTo;

const InstallStateId = installer.ffi.InstallStateId;
// ── Trampoline ────────────────────────────────────────────────────────────────
pub fn stateStart(machine: *InstallerMachine) InstallerError!void {
    var state = InstallStateId.verifying;
    while (state != .done) {
        try machine.enter(state);
        state = switch (state) {
            .verifying => try stateVerifying(machine),
            .check_space => try stateCheckSpace(machine),
            .open_repo => try stateOpenRepo(machine),
            .check_installed => try stateCheckInstalled(machine),
            .write_database => try stateWriteDatabase(machine),
            .remap_paths => try stateRemapPaths(machine),
            .process_db_files => try stateProcessDbFiles(machine),
            .commit => try stateCommit(machine),
            .checkout_binaries_files => try stateCheckoutBinariesFiles(machine),
            .prepare_config_staging => try statePrepareConfigStaging(machine),
            .atomic_swap => try stateAtomicSwap(machine),
            .swap_config_files => try stateSwapConfigFiles(machine),
            .cleanup => try stateCleanupStaging(machine),
            .done, .failed => unreachable,
        };
    }
    if (machine.repo) |repo| {
        var objects_total: c_libs.gint = 0;
        var objects_pruned: c_libs.gint = 0;
        var pruned_size: c_libs.guint64 = 0;
        _ = c_libs.ostree_repo_prune(repo, c_libs.OSTREE_REPO_PRUNE_FLAGS_REFS_ONLY, -1, &objects_total, &objects_pruned, &pruned_size, null, null);
    }

    try machine.enter(.done);
}

// ── InstallerFSM states ───────────────────────────────────────────────────────
fn stateVerifying(machine: *InstallerMachine) InstallerError!InstallStateId {
    for (machine.data.packages) |entry| try machine.check(std.Io.Dir.accessAbsolute(machine.io, std.mem.span(entry.temp_path), .{}), InstallerError.PathNotFound);

    try machine.check(std.Io.Dir.accessAbsolute(machine.io, std.mem.span(machine.data.root_path), .{}), InstallerError.PathNotFound);
    try machine.check(std.Io.Dir.accessAbsolute(machine.io, std.mem.span(machine.data.repo_path), .{}), InstallerError.PathNotFound);

    const prefix_directory = try machine.check(std.fs.path.join(machine.allocator, &.{ std.mem.span(machine.data.root_path), PREFIX }), InstallerError.AllocZFailed);
    defer machine.allocator.free(prefix_directory);

    try machine.check(std.Io.Dir.accessAbsolute(machine.io, prefix_directory, .{}), InstallerError.PathNotFound);

    machine.resetRetries();
    return .check_space;
}

fn stateCheckSpace(machine: *InstallerMachine) InstallerError!InstallStateId {
    var new_packages_size: u64 = 0;
    for (machine.data.packages) |entry| new_packages_size += try machine.check(dirSize(machine, std.mem.span(entry.temp_path)), InstallerError.CheckSpaceFailed);

    const prefix_path = try machine.prefixPathZ();
    defer machine.allocator.free(prefix_path);

    const existing_prefix_size = dirSize(machine, prefix_path) catch 0;
    const required = existing_prefix_size + new_packages_size * 2;

    var stat: c_libs.struct_statvfs = undefined;
    if (c_libs.statvfs(machine.data.root_path, &stat) != 0) {
        stateFailed(machine);
        return InstallerError.CheckSpaceFailed;
    }

    const available: u64 = @as(u64, @intCast(stat.f_bavail)) * @as(u64, @intCast(stat.f_bsize));
    if (required > available) {
        stateFailed(machine);
        return InstallerError.NotEnoughSpace;
    }

    machine.resetRetries();
    return .open_repo;
}

fn stateOpenRepo(machine: *InstallerMachine) InstallerError!InstallStateId {
    const gfile = c_libs.g_file_new_for_path(machine.data.repo_path);
    defer c_libs.g_object_unref(@ptrCast(gfile));

    const repo = c_libs.ostree_repo_new(gfile);

    if (c_libs.ostree_repo_open(repo, machine.cancellable, &machine.gerror) == 0) {
        c_libs.g_object_unref(repo);
        return machine.retry(.open_repo);
    }
    machine.repo = repo;

    try machine.gcheck(c_libs.ostree_repo_prepare_transaction(repo, null, machine.cancellable, &machine.gerror), error.RepoTransactionFailed);

    var previos_mtree: ?*c_libs.OstreeMutableTree = null;
    if (c_libs.ostree_repo_resolve_rev(repo, machine.data.branch, 0, &machine.previous_commit_checksum, null) != 0) {
        previos_mtree = c_libs.ostree_mutable_tree_new_from_commit(repo, machine.previous_commit_checksum, &machine.gerror);
        if (previos_mtree == null) {
            if (machine.gerror != null) {
                stateFailed(machine);
                return InstallerError.RepoTransactionFailed;
            }
            previos_mtree = c_libs.ostree_mutable_tree_new();
        }
    } else {
        previos_mtree = c_libs.ostree_mutable_tree_new();
    }

    machine.mtree = previos_mtree;

    machine.resetRetries();
    return .check_installed;
}

fn stateCheckInstalled(machine: *InstallerMachine) InstallerError!InstallStateId {
    const body = try loadCommitBody(machine, machine.previous_commit_checksum);
    defer machine.allocator.free(body);

    const current_name = machine.data.packages[machine.current_package_index].package.meta.name;

    if (try machine.check(find(body, current_name, machine.allocator), InstallerError.AllocZFailed) != null) {
        stateFailed(machine);
        return InstallerError.AlreadyInstalled;
    }

    machine.resetRetries();
    return .write_database;
}

fn stateWriteDatabase(machine: *InstallerMachine) InstallerError!InstallStateId {
    const current_install_entry = machine.data.packages[machine.current_package_index];

    const staged_database_dir_path = try machine.check(std.fs.path.join(machine.allocator, &.{ std.mem.span(current_install_entry.temp_path), DB_RELATIVE_PATH }), InstallerError.AllocZFailed);
    defer machine.allocator.free(staged_database_dir_path);

    try machine.check(std.Io.Dir.cwd().createDirPath(machine.io, staged_database_dir_path), InstallerError.AllocZFailed);

    var file_map = data.FileMap.init(machine.allocator);
    defer data.freeFileMap(&file_map, machine.allocator);

    collectFileChecksums(machine, &file_map) catch |err| {
        if (err == error.Cancelled) return error.Cancelled;
        stateFailed(machine);
        return InstallerError.CollectFileChecksumsFailed;
    };

    data.writePackage(staged_database_dir_path, std.mem.span(current_install_entry.checksum), current_install_entry.package.meta, file_map, machine.allocator) catch {
        stateFailed(machine);
        return InstallerError.WriteDatabaseFailed;
    };

    machine.resetRetries();
    return .remap_paths;
}

fn stateRemapPaths(machine: *InstallerMachine) InstallerError!InstallStateId {
    const current_entry = machine.data.packages[machine.current_package_index];
    const temp_path = std.mem.span(current_entry.temp_path);

    var temp_dir = try machine.check(
        std.Io.Dir.openDirAbsolute(machine.io, temp_path, .{ .iterate = true }),
        InstallerError.WriteFilesFailed,
    );
    defer temp_dir.close(machine.io);

    var to_remap = std.ArrayList([]const u8).empty;
    defer {
        for (to_remap.items) |name| machine.allocator.free(name);
        to_remap.deinit(machine.allocator);
    }

    var walker = try machine.check(temp_dir.walk(machine.allocator), InstallerError.WriteFilesFailed);
    defer walker.deinit();

    while (try machine.check(walker.next(machine.io), InstallerError.WriteFilesFailed)) |entry| {
        if (entry.kind != .directory) continue;
        if (std.mem.indexOfScalar(u8, entry.path, '/') != null) continue;
        if (std.mem.eql(u8, entry.path, PREFIX)) continue;
        if (std.mem.eql(u8, entry.path, "etc")) continue;
        try machine.check(
            to_remap.append(machine.allocator, try machine.allocator.dupe(u8, entry.path)),
            InstallerError.AllocZFailed,
        );
    }

    for (to_remap.items) |name| {
        const src = try machine.check(
            std.fs.path.joinZ(machine.allocator, &.{ temp_path, name }),
            InstallerError.AllocZFailed,
        );
        defer machine.allocator.free(src);

        const dst = try machine.check(
            std.fs.path.joinZ(machine.allocator, &.{ temp_path, PREFIX, name }),
            InstallerError.AllocZFailed,
        );
        defer machine.allocator.free(dst);

        const rename_result = std.os.linux.syscall4(
            .renameat,
            @bitCast(@as(isize, std.c.AT.FDCWD)),
            @intFromPtr(src.ptr),
            @bitCast(@as(isize, std.c.AT.FDCWD)),
            @intFromPtr(dst.ptr),
        );
        if (std.os.linux.errno(rename_result) != .SUCCESS) {
            try machine.check(mergeDirs(src, dst, machine.allocator), InstallerError.WriteFilesFailed);
        }
    }

    machine.resetRetries();
    return .process_db_files;
}

fn stateProcessDbFiles(machine: *InstallerMachine) InstallerError!InstallStateId {
    const repo = try machine.unwrap(machine.repo, InstallerError.RepoOpenFailed);
    const mtree = try machine.unwrap(machine.mtree, InstallerError.RepoOpenFailed);

    const current_install_entry = machine.data.packages[machine.current_package_index];

    const temp_path_c = try machine.check(machine.allocator.dupeZ(u8, std.mem.span(current_install_entry.temp_path)), InstallerError.AllocZFailed);
    defer machine.allocator.free(temp_path_c);

    if (c_libs.ostree_repo_write_dfd_to_mtree(repo, std.c.AT.FDCWD, temp_path_c.ptr, mtree, null, machine.cancellable, &machine.gerror) == 0) {
        stateFailed(machine);
        return InstallerError.WriteFilesFailed;
    }

    machine.current_package_index += 1;
    if (machine.current_package_index < machine.data.packages.len) {
        machine.resetRetries();
        return .check_installed;
    }

    machine.resetRetries();
    return .commit;
}

fn stateCommit(machine: *InstallerMachine) InstallerError!InstallStateId {
    const repo = try machine.unwrap(machine.repo, InstallerError.RepoOpenFailed);
    const mtree = try machine.unwrap(machine.mtree, InstallerError.PackageNotFound);

    var body = try loadCommitBody(machine, machine.previous_commit_checksum);
    for (machine.data.packages) |entry| {
        const new_body = append(body, entry.package.meta.name, std.mem.span(entry.checksum), machine.allocator) catch return InstallerError.AllocZFailed;
        machine.allocator.free(body);
        body = new_body;
    }
    defer machine.allocator.free(body);

    const body_c = try machine.check(machine.allocator.dupeZ(u8, body), InstallerError.AllocZFailed);
    defer machine.allocator.free(body_c);

    var mtree_root: ?*c_libs.GFile = null;
    defer if (mtree_root) |root| c_libs.g_object_unref(root);

    if (c_libs.ostree_repo_write_mtree(repo, mtree, &mtree_root, machine.cancellable, &machine.gerror) == 0)
        return machine.retry(.commit);

    var subject_buf = std.Io.Writer.Allocating.init(machine.allocator);
    defer subject_buf.deinit();

    try machine.check(subject_buf.writer.writeAll("install:"), InstallerError.AllocZFailed);
    for (machine.data.packages, 0..) |entry, i| {
        try machine.check(subject_buf.writer.print("{s}{s} {s}", .{ if (i == 0) " " else ", ", entry.package.meta.name, entry.package.meta.version }), InstallerError.AllocZFailed);
    }

    const subject_c = try machine.check(machine.allocator.dupeZ(u8, subject_buf.written()), InstallerError.AllocZFailed);
    defer machine.allocator.free(subject_c);

    if (c_libs.ostree_repo_write_commit(repo, machine.previous_commit_checksum, subject_c.ptr, body_c.ptr, null, @ptrCast(mtree_root), &machine.commit_checksum, machine.cancellable, &machine.gerror) == 0) return machine.retry(.commit);

    c_libs.ostree_repo_transaction_set_ref(repo, null, machine.data.branch, machine.commit_checksum);

    if (c_libs.ostree_repo_commit_transaction(repo, null, machine.cancellable, &machine.gerror) == 0) return machine.retry(.commit);

    machine.resetRetries();
    return .checkout_binaries_files;
}

fn stateCheckoutBinariesFiles(machine: *InstallerMachine) InstallerError!InstallStateId {
    const repo = try machine.unwrap(machine.repo, InstallerError.RepoOpenFailed);
    const estimated = estimateCheckoutSize(machine) catch 0;
    if (machine.gerror) |err| {
        c_libs.g_error_free(err);
        machine.gerror = null;
    }

    var buf: [256]u8 = undefined;
    var ts: std.os.linux.timespec = undefined;
    _ = std.os.linux.clock_gettime(std.os.linux.CLOCK.REALTIME, &ts);
    const timestamp: i64 = @as(i64, ts.sec) * 1000 + @divTrunc(@as(i64, ts.nsec), 1_000_000);
    var stat: c_libs.struct_statvfs = undefined;
    if (c_libs.statvfs(machine.data.root_path, &stat) == 0 and estimated * 2 > @as(u64, stat.f_bavail) * @as(u64, stat.f_bsize)) {
        stateFailed(machine);
        return InstallerError.NotEnoughSpace;
    }

    const temp_folder_name = try machine.check(std.fmt.bufPrintZ(&buf, "{s}-install-{d}", .{ PREFIX, timestamp }), error.AllocZFailed);

    const staging_path_c = try machine.check(std.fs.path.joinZ(machine.allocator, &.{ std.mem.span(machine.data.root_path), temp_folder_name }), InstallerError.AllocZFailed);
    machine.staging_path_c = staging_path_c;

    var options = std.mem.zeroes(c_libs.OstreeRepoCheckoutAtOptions);
    options.mode = c_libs.OSTREE_REPO_CHECKOUT_MODE_NONE;
    options.overwrite_mode = c_libs.OSTREE_REPO_CHECKOUT_OVERWRITE_UNION_FILES;

    if (machine.commit_checksum == null) {
        stateFailed(machine);
        return InstallerError.CheckoutFailed;
    } else if (c_libs.ostree_repo_checkout_at(repo, &options, std.c.AT.FDCWD, staging_path_c, machine.commit_checksum, machine.cancellable, &machine.gerror) == 0) {
        const _io = std.Io.Threaded.global_single_threaded.io();
        std.Io.Dir.cwd().deleteTree(_io, staging_path_c) catch {};
        machine.allocator.free(staging_path_c);
        machine.staging_path_c = null;
        if (machine.gerror) |err| {
            c_libs.g_error_free(err);
            machine.gerror = null;
        }
        machine.retries += 1;
        if (machine.exhausted()) {
            stateFailed(machine);
            return InstallerError.CheckoutFailed;
        }
        return .checkout_binaries_files;
    }

    machine.resetRetries();
    return .prepare_config_staging;
}

fn stateAtomicSwap(machine: *InstallerMachine) InstallerError!InstallStateId {
    const staging_path = try machine.unwrap(machine.staging_path_c, InstallerError.CheckoutFailed);

    const root_prefix_path_c = try machine.check(std.fs.path.joinZ(machine.allocator, &.{ std.mem.span(machine.data.root_path), PREFIX }), InstallerError.AllocZFailed);
    defer machine.allocator.free(root_prefix_path_c);

    const staging_prefix_path_c = try machine.check(std.fs.path.joinZ(machine.allocator, &.{ staging_path, PREFIX }), InstallerError.AllocZFailed);
    defer machine.allocator.free(staging_prefix_path_c);

    const result = std.os.linux.syscall5(.renameat2, @bitCast(@as(isize, std.os.linux.AT.FDCWD)), @intFromPtr(staging_prefix_path_c.ptr), @bitCast(@as(isize, std.os.linux.AT.FDCWD)), @intFromPtr(root_prefix_path_c.ptr), 2);

    if (std.os.linux.errno(result) != .SUCCESS) {
        try machine.check(std.Io.Dir.cwd().deleteTree(machine.io, staging_path), InstallerError.CheckoutFailed);
        stateFailed(machine);
        return InstallerError.CheckoutFailed;
    }

    machine.resetRetries();
    return .swap_config_files;
}

fn statePrepareConfigStaging(machine: *InstallerMachine) InstallerError!InstallStateId {
    const staging_path = try machine.unwrap(machine.staging_path_c, InstallerError.CheckoutFailed);

    const staging_etc = try machine.check(
        std.fs.path.joinZ(machine.allocator, &.{ staging_path, "etc" }),
        InstallerError.AllocZFailed,
    );
    defer machine.allocator.free(staging_etc);

    std.Io.Dir.accessAbsolute(machine.io, staging_etc, .{}) catch {
        machine.resetRetries();
        return .atomic_swap;
    };

    const etc_new = try machine.check(
        std.fs.path.joinZ(machine.allocator, &.{ staging_path, "etc-new" }),
        InstallerError.AllocZFailed,
    );
    defer machine.allocator.free(etc_new);

    try machine.check(std.Io.Dir.cwd().createDirPath(machine.io, etc_new), InstallerError.WriteConfigFailed);

    const root_etc = try machine.check(
        std.fs.path.joinZ(machine.allocator, &.{ std.mem.span(machine.data.root_path), "etc" }),
        InstallerError.AllocZFailed,
    );
    defer machine.allocator.free(root_etc);

    std.Io.Dir.accessAbsolute(machine.io, root_etc, .{}) catch {};
    mirrorDir(machine, root_etc, etc_new) catch {
        stateFailed(machine);
        return InstallerError.WriteConfigFailed;
    };
    overlayDir(machine, staging_etc, etc_new) catch {
        stateFailed(machine);
        return InstallerError.WriteConfigFailed;
    };

    machine.resetRetries();
    return .atomic_swap;
}

fn stateSwapConfigFiles(machine: *InstallerMachine) InstallerError!InstallStateId {
    const staging_path = try machine.unwrap(machine.staging_path_c, InstallerError.CheckoutFailed);

    const etc_new = try machine.check(
        std.fs.path.joinZ(machine.allocator, &.{ staging_path, "etc-new" }),
        InstallerError.AllocZFailed,
    );
    defer machine.allocator.free(etc_new);

    std.Io.Dir.accessAbsolute(machine.io, etc_new, .{}) catch {
        machine.resetRetries();
        return .cleanup;
    };

    const root_etc = try machine.check(
        std.fs.path.joinZ(machine.allocator, &.{ std.mem.span(machine.data.root_path), "etc" }),
        InstallerError.AllocZFailed,
    );
    defer machine.allocator.free(root_etc);

    const result = std.os.linux.syscall5(.renameat2, @bitCast(@as(isize, std.c.AT.FDCWD)), @intFromPtr(etc_new.ptr), @bitCast(@as(isize, std.c.AT.FDCWD)), @intFromPtr(root_etc.ptr), 2);
    if (std.os.linux.errno(result) != .SUCCESS) {
        stateFailed(machine);
        return InstallerError.WriteConfigFailed;
    }

    machine.resetRetries();
    return .cleanup;
}

fn stateCleanupStaging(machine: *InstallerMachine) InstallerError!InstallStateId {
    const staging_path = try machine.unwrap(machine.staging_path_c, InstallerError.CheckoutFailed);

    try machine.check(std.Io.Dir.cwd().deleteTree(machine.io, staging_path), InstallerError.CheckoutFailed);
    machine.allocator.free(staging_path);
    machine.staging_path_c = null;

    return .done;
}

pub fn stateFailed(machine: *InstallerMachine) void {
    if (machine.stack.items.len > 0 and machine.stack.getLast() == .failed) return;
    var abort_err: ?*c_libs.GError = null;
    defer if (abort_err) |err| c_libs.g_error_free(err);

    if (machine.staging_path_c) |staging| {
        std.Io.Dir.cwd().deleteTree(machine.io, staging) catch {};
        machine.allocator.free(staging);
        machine.staging_path_c = null;
    }

    if (machine.repo) |repo| {
        _ = c_libs.ostree_repo_abort_transaction(repo, null, &abort_err);

        if (machine.commit_checksum != null) _ = c_libs.ostree_repo_set_ref_immediate(repo, null, machine.data.branch, machine.previous_commit_checksum, null, null);
    }

    machine.stack.append(machine.allocator, .failed) catch {};
    machine.report(.failed);
}
