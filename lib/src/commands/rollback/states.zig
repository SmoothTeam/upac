// ── Imports ─────────────────────────────────────────────────────────────────────
const rollback = @import("rollback.zig");
const std = rollback.std;
const c_libs = rollback.c_libs;

const PREFIX = rollback.PREFIX;

const RollbackMachine = rollback.RollbackMachine;
const RollbackError = rollback.RollbackError;

const RollbackStateId = rollback.ffi.RollbackStateId;

const utils = @import("utils.zig");
const resolveStagingDir = utils.resolveStagingDir;
const resolveRootDir = utils.resolveRootDir;

// ── Trampoline ────────────────────────────────────────────────────────────────
pub fn stateStart(machine: *RollbackMachine) RollbackError!void {
    var state = RollbackStateId.verifying;
    while (state != .done) {
        try machine.enter(state);
        state = switch (state) {
            .verifying => try stateVerifying(machine),
            .open_repo => try stateOpenRepo(machine),
            .resolve_commit => try stateResolveCommit(machine),
            .checkout_staging => try stateCheckoutStaging(machine),
            .atomic_swap => try stateAtomicSwap(machine),
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
    defer c_libs.g_object_unref(@ptrCast(gfile));

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
    const repo = try machine.unwrap(machine.repo, error.RepoOpenFailed);

    var resolved: [*c]u8 = null;
    try machine.gcheck(c_libs.ostree_repo_resolve_rev(repo, machine.data.commit_hash, 0, &resolved, &machine.gerror), error.CommitNotFound);

    var has_object: c_libs.gboolean = 0;
    _ = c_libs.ostree_repo_has_object(repo, c_libs.OSTREE_OBJECT_TYPE_COMMIT, resolved, &has_object, machine.cancellable, null);
    if (has_object == 0) {
        c_libs.g_free(@ptrCast(resolved));
        stateFailed(machine);
        return error.CommitNotFound;
    }

    machine.resolved_checksum = resolved;

    machine.resetRetries();
    return .checkout_staging;
}

fn stateCheckoutStaging(machine: *RollbackMachine) RollbackError!RollbackStateId {
    const repo = try machine.unwrap(machine.repo, error.RepoOpenFailed);
    const resolved_checksum = try machine.unwrap(machine.resolved_checksum, error.CommitNotFound);

    machine.staging_path_c = try resolveStagingDir(std.mem.span(machine.data.root_path), machine.allocator);
    const staging_path_c = try machine.unwrap(machine.staging_path_c, error.StagingFailed);

    try machine.check(std.Io.Dir.createDirAbsolute(machine.io, staging_path_c, .default_dir), RollbackError.StagingFailed);

    var options = std.mem.zeroes(c_libs.OstreeRepoCheckoutAtOptions);
    options.mode = c_libs.OSTREE_REPO_CHECKOUT_MODE_NONE;
    options.overwrite_mode = c_libs.OSTREE_REPO_CHECKOUT_OVERWRITE_ADD_FILES;
    options.no_copy_fallback = 0;

    if (c_libs.ostree_repo_checkout_at(repo, &options, std.c.AT.FDCWD, staging_path_c, resolved_checksum, machine.cancellable, &machine.gerror) == 0) {
        try machine.check(std.Io.Dir.cwd().deleteTree(machine.io, staging_path_c), RollbackError.RollbackFailed);

        machine.allocator.free(staging_path_c);
        machine.staging_path_c = null;

        return machine.retry(.checkout_staging);
    }

    machine.resetRetries();
    return .atomic_swap;
}

fn stateAtomicSwap(machine: *RollbackMachine) RollbackError!RollbackStateId {
    const staging_path_c = try machine.unwrap(machine.staging_path_c, error.StagingFailed);

    const root_prefix_path = try resolveRootDir(std.mem.span(machine.data.root_path), machine.allocator);
    defer machine.allocator.free(root_prefix_path);

    const staging_prefix_path = try resolveRootDir(staging_path_c, machine.allocator);
    defer machine.allocator.free(staging_prefix_path);

    const result = std.os.linux.syscall5(.renameat2, @bitCast(@as(isize, std.os.linux.AT.FDCWD)), @intFromPtr(staging_prefix_path.ptr), @bitCast(@as(isize, std.os.linux.AT.FDCWD)), @intFromPtr(root_prefix_path.ptr), 2);

    if (std.os.linux.errno(result) != .SUCCESS) {
        stateFailed(machine);
        return error.SwapFailed;
    }

    machine.resetRetries();
    return .cleanup_staging;
}

fn stateCleanupStaging(machine: *RollbackMachine) RollbackError!RollbackStateId {
    const staging_path_c = try machine.unwrap(machine.staging_path_c, error.StagingFailed);

    try machine.check(std.Io.Dir.cwd().deleteTree(machine.io, staging_path_c), RollbackError.CleanupFailed);

    machine.allocator.free(staging_path_c);
    machine.staging_path_c = null;

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
    if (machine.staging_path_c) |staging_path| {
        std.Io.Dir.cwd().deleteTree(machine.io, staging_path) catch {};
        machine.allocator.free(staging_path);
        machine.staging_path_c = null;
    }
    machine.stack.append(machine.allocator, .failed) catch {};
    machine.report(.failed);
}
