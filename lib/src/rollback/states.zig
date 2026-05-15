// ── Imports ─────────────────────────────────────────────────────────────────────
const rollback = @import("rollback.zig");
const std = rollback.std;
const c_libs = rollback.c_libs;

const PREFIX = rollback.PREFIX;
const CONFIG_DIR = rollback.CONFIG_DIR;
const CONFIG_STAGING_DIR = rollback.CONFIG_STAGING_DIR;

const RollbackMachine = rollback.RollbackMachine;
const RollbackError = rollback.RollbackError;

const RollbackStateId = rollback.ffi.RollbackStateId;

const utils = @import("utils.zig");
const resolveStagingDir = utils.resolveStagingDir;
const resolveRootDir = utils.resolveRootDir;
const mirrorDir = utils.mirrorDir;
const overlayDir = utils.overlayDir;

// ── Trampoline ────────────────────────────────────────────────────────────────
pub fn stateStart(machine: *RollbackMachine) RollbackError!void {
    var state = RollbackStateId.verifying;
    while (state != .done) {
        try machine.enter(state);
        state = switch (state) {
            .verifying => try stateVerifying(machine),
            .open_repo => try stateOpenRepo(machine),
            .resolve_commit => try stateResolveCommit(machine),
            .checkout_staging => try stateCheckoutBinariesFiles(machine),
            .prepare_config_staging => try statePrepareConfigStaging(machine),
            .swap_binaries_files => try stateSwapBinariesFiles(machine),
            .swap_config_files => try stateSwapConfigFiles(machine),
            .cleanup_staging => try stateCleanupStaging(machine),
            .update_ref => try stateUpdateRef(machine),
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

// ── States ────────────────────────────────────────────────────────────────────
fn stateVerifying(machine: *RollbackMachine) RollbackError!RollbackStateId {
    try machine.check(std.Io.Dir.accessAbsolute(machine.io, std.mem.span(machine.data.root_path), .{}), RollbackError.PathNotFound);
    try machine.check(std.Io.Dir.accessAbsolute(machine.io, std.mem.span(machine.data.repo_path), .{}), RollbackError.RepoOpenFailed);

    const prefix_directory = try machine.check(std.fs.path.join(machine.allocator, &.{ std.mem.span(machine.data.root_path), PREFIX }), RollbackError.AllocZFailed);
    defer machine.allocator.free(prefix_directory);

    try machine.check(std.Io.Dir.accessAbsolute(machine.io, prefix_directory, .{}), RollbackError.PathNotFound);

    machine.resetRetries();
    return .open_repo;
}

fn stateOpenRepo(machine: *RollbackMachine) RollbackError!RollbackStateId {
    const gfile = c_libs.g_file_new_for_path(machine.data.repo_path);
    defer c_libs.g_object_unref(gfile);

    const repo = c_libs.ostree_repo_new(gfile);
    if (c_libs.ostree_repo_open(repo, machine.cancellable, &machine.gerror) == 0) {
        c_libs.g_object_unref(repo);
        return machine.retry(.open_repo);
    }

    machine.repo = repo;

    machine.resetRetries();
    return .resolve_commit;
}

fn stateResolveCommit(machine: *RollbackMachine) RollbackError!RollbackStateId {
    var resolved: [*c]u8 = null;
    var has_object: c_libs.gboolean = 0;

    const repo = try machine.unwrap(machine.repo, error.RepoOpenFailed);

    try machine.gcheck(c_libs.ostree_repo_resolve_rev(repo, machine.data.commit_hash, 0, &resolved, &machine.gerror), error.CommitNotFound);

    _ = c_libs.ostree_repo_has_object(repo, c_libs.OSTREE_OBJECT_TYPE_COMMIT, resolved, &has_object, machine.cancellable, null);
    try machine.gcheck(has_object, error.CommitNotFound);

    machine.resolved_checksum = resolved;

    machine.resetRetries();
    return .checkout_staging;
}

fn stateCheckoutBinariesFiles(machine: *RollbackMachine) RollbackError!RollbackStateId {
    var options = std.mem.zeroes(c_libs.OstreeRepoCheckoutAtOptions);
    options.mode = c_libs.OSTREE_REPO_CHECKOUT_MODE_NONE;
    options.overwrite_mode = c_libs.OSTREE_REPO_CHECKOUT_OVERWRITE_ADD_FILES;
    options.no_copy_fallback = 0;

    const staging_prefix_path_c = try resolveStagingDir(std.mem.span(machine.data.root_path), machine.allocator);
    machine.staging_prefix_path_c = staging_prefix_path_c;

    const repo = try machine.unwrap(machine.repo, error.RepoOpenFailed);
    const resolved_checksum = try machine.unwrap(machine.resolved_checksum, error.CommitNotFound);

    try machine.check(std.Io.Dir.createDirAbsolute(machine.io, staging_prefix_path_c, .default_dir), RollbackError.StagingFailed);

    if (c_libs.ostree_repo_checkout_at(repo, &options, std.c.AT.FDCWD, staging_prefix_path_c, resolved_checksum, machine.cancellable, &machine.gerror) == 0) {
        try machine.check(std.Io.Dir.cwd().deleteTree(machine.io, staging_prefix_path_c), RollbackError.RollbackFailed);

        machine.allocator.free(staging_prefix_path_c);
        machine.staging_prefix_path_c = null;

        return machine.retry(.checkout_staging);
    }

    machine.resetRetries();
    return .prepare_config_staging;
}

fn statePrepareConfigStaging(machine: *RollbackMachine) RollbackError!RollbackStateId {
    const staging_prefix_path = try machine.unwrap(machine.staging_prefix_path_c, RollbackError.StagingFailed);

    const staging_etc = try machine.check(
        std.fs.path.joinZ(machine.allocator, &.{ staging_prefix_path, CONFIG_DIR }),
        RollbackError.AllocZFailed,
    );
    defer machine.allocator.free(staging_etc);

    const new_config_path = try machine.check(std.fs.path.joinZ(machine.allocator, &.{ staging_prefix_path, CONFIG_STAGING_DIR }), RollbackError.AllocZFailed);
    machine.staging_config_path_c = new_config_path;

    const old_config_path = try machine.check(std.fs.path.joinZ(machine.allocator, &.{ std.mem.span(machine.data.root_path), CONFIG_DIR }), RollbackError.AllocZFailed);
    defer machine.allocator.free(old_config_path);

    std.Io.Dir.accessAbsolute(machine.io, staging_etc, .{}) catch {
        machine.resetRetries();
        return .swap_binaries_files;
    };

    try machine.check(std.Io.Dir.cwd().createDirPath(machine.io, new_config_path), RollbackError.StagingFailed);

    mirrorDir(machine, old_config_path, new_config_path) catch {
        stateFailed(machine);
        return RollbackError.StagingFailed;
    };
    overlayDir(machine, staging_etc, new_config_path) catch {
        stateFailed(machine);
        return RollbackError.StagingFailed;
    };

    machine.resetRetries();
    return .swap_binaries_files;
}

fn stateSwapBinariesFiles(machine: *RollbackMachine) RollbackError!RollbackStateId {
    const staging_prefix_path_c = try machine.unwrap(machine.staging_prefix_path_c, error.StagingFailed);

    const root_prefix_path = try resolveRootDir(std.mem.span(machine.data.root_path), machine.allocator);
    defer machine.allocator.free(root_prefix_path);

    const staging_prefix_path = try resolveRootDir(staging_prefix_path_c, machine.allocator);
    defer machine.allocator.free(staging_prefix_path);

    const prefix_swap_result = std.os.linux.syscall5(.renameat2, @bitCast(@as(isize, std.os.linux.AT.FDCWD)), @intFromPtr(staging_prefix_path.ptr), @bitCast(@as(isize, std.os.linux.AT.FDCWD)), @intFromPtr(root_prefix_path.ptr), 2);

    if (std.os.linux.errno(prefix_swap_result) != .SUCCESS) {
        stateFailed(machine);
        return error.SwapFailed;
    }

    machine.resetRetries();
    return .swap_config_files;
}

fn stateSwapConfigFiles(machine: *RollbackMachine) RollbackError!RollbackStateId {
    const new_config_path = machine.staging_config_path_c orelse {
        machine.resetRetries();
        return .cleanup_staging;
    };

    const old_config_path = try machine.check(std.fs.path.joinZ(machine.allocator, &.{ std.mem.span(machine.data.root_path), CONFIG_DIR }), RollbackError.AllocZFailed);
    defer machine.allocator.free(old_config_path);

    const config_swap_result = std.os.linux.syscall5(.renameat2, @bitCast(@as(isize, std.c.AT.FDCWD)), @intFromPtr(new_config_path.ptr), @bitCast(@as(isize, std.c.AT.FDCWD)), @intFromPtr(old_config_path.ptr), 2);
    if (std.os.linux.errno(config_swap_result) != .SUCCESS) {
        stateFailed(machine);
        return RollbackError.SwapFailed;
    }

    machine.resetRetries();
    return .cleanup_staging;
}

fn stateCleanupStaging(machine: *RollbackMachine) RollbackError!RollbackStateId {
    const staging_prefix_path_c = try machine.unwrap(machine.staging_prefix_path_c, error.StagingFailed);

    try machine.check(std.Io.Dir.cwd().deleteTree(machine.io, staging_prefix_path_c), RollbackError.CleanupFailed);

    machine.allocator.free(staging_prefix_path_c);
    machine.staging_prefix_path_c = null;

    if (machine.staging_config_path_c) |path| {
        machine.allocator.free(path);
        machine.staging_config_path_c = null;
    }

    machine.resetRetries();
    return .update_ref;
}

fn stateUpdateRef(machine: *RollbackMachine) RollbackError!RollbackStateId {
    const repo = try machine.unwrap(machine.repo, error.RepoOpenFailed);
    const resolved_checksum = try machine.unwrap(machine.resolved_checksum, error.CommitNotFound);

    if (c_libs.ostree_repo_prepare_transaction(repo, null, machine.cancellable, &machine.gerror) == 0) return machine.retry(.update_ref);

    c_libs.ostree_repo_transaction_set_ref(repo, null, machine.data.branch, resolved_checksum);

    if (c_libs.ostree_repo_commit_transaction(repo, null, machine.cancellable, &machine.gerror) == 0) {
        _ = c_libs.ostree_repo_abort_transaction(repo, machine.cancellable, null);
        return machine.retry(.update_ref);
    }

    machine.resetRetries();
    return .done;
}

pub fn stateFailed(machine: *RollbackMachine) void {
    if (machine.staging_prefix_path_c) |staging_path| {
        std.Io.Dir.cwd().deleteTree(machine.io, staging_path) catch {};
        machine.allocator.free(staging_path);
        machine.staging_prefix_path_c = null;
    }
    if (machine.staging_config_path_c) |path| {
        machine.allocator.free(path);
        machine.staging_config_path_c = null;
    }
    machine.stack.append(machine.allocator, .failed) catch {};
    machine.report(.failed);
}
