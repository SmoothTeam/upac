const std = @import("std");

const c_libs = @import("c-libs");

const update = @import("../update.zig");
const UpdateMachine = update.UpdateMachine;
const UpdateError = update.UpdateError;

const utils = @import("utils.zig");
const resolveTempDir = utils.resolveTempDir;

// ── CheckoutState ─────────────────────────────────────────────────────────────
const CheckoutState = enum {
    open_repo,
    resolve_commit,
    checkout,
    done,
};

// ── CheckoutMachine ───────────────────────────────────────────────────────────
pub const CheckoutMachine = struct {
    updater: *UpdateMachine,

    repo: ?*c_libs.OstreeRepo = null,

    commit_checksum: [*c]u8 = null,

    temp_prefix_path: ?[:0]u8 = null,

    fn stateFailed(self: *CheckoutMachine, err: UpdateError) UpdateError {
        if (self.repo) |repo| {
            c_libs.g_object_unref(repo);
            self.repo = null;
        }
        if (self.commit_checksum != null) {
            c_libs.g_free(self.commit_checksum);
            self.commit_checksum = null;
        }
        if (self.temp_prefix_path) |path| {
            std.Io.Dir.cwd().deleteTree(self.updater.io, path) catch {};
            self.updater.allocator.free(path);
            self.temp_prefix_path = null;
            self.updater.temp_prefix_path = null;
        }
        return err;
    }
};

// ── Trampoline ────────────────────────────────────────────────────────────────
pub fn run(machine: *UpdateMachine) UpdateError!void {
    var checkout_machine = CheckoutMachine{ .updater = machine };

    var state = CheckoutState.open_repo;
    while (state != .done) {
        if (machine.cancellable) |cancellable| if (c_libs.g_cancellable_is_cancelled(cancellable) != 0) return checkout_machine.stateFailed(UpdateError.Cancelled);

        state = switch (state) {
            .open_repo => try stateOpenRepo(&checkout_machine),
            .resolve_commit => try stateResolveCommit(&checkout_machine),
            .checkout => try stateCheckout(&checkout_machine),
            .done => unreachable,
        };
    }
}

// ── States ────────────────────────────────────────────────────────────────────
fn stateOpenRepo(machine: *CheckoutMachine) UpdateError!CheckoutState {
    const gfile = c_libs.g_file_new_for_path(machine.updater.data.repo_path);
    defer c_libs.g_object_unref(gfile);

    const repo = c_libs.ostree_repo_new(gfile);
    if (c_libs.ostree_repo_open(repo, machine.updater.cancellable, &machine.updater.gerror) == 0) {
        c_libs.g_object_unref(repo);
        return machine.stateFailed(UpdateError.RepoOpenFailed);
    }
    machine.repo = repo;

    return .resolve_commit;
}

fn stateResolveCommit(machine: *CheckoutMachine) UpdateError!CheckoutState {
    const repo = machine.repo orelse return machine.stateFailed(UpdateError.RepoOpenFailed);

    if (c_libs.ostree_repo_resolve_rev(repo, machine.updater.data.branch, 0, &machine.commit_checksum, &machine.updater.gerror) == 0) return machine.stateFailed(UpdateError.CommitNotFound);
    if (machine.commit_checksum == null) return machine.stateFailed(UpdateError.CommitNotFound);

    return .checkout;
}

fn stateCheckout(machine: *CheckoutMachine) UpdateError!CheckoutState {
    const repo = machine.repo orelse return machine.stateFailed(UpdateError.RepoOpenFailed);
    const checksum = machine.commit_checksum orelse return machine.stateFailed(UpdateError.CommitNotFound);

    const root_path = std.mem.span(machine.updater.data.root_path);

    const temp_prefix_path = resolveTempDir(root_path, machine.updater.allocator) catch return machine.stateFailed(UpdateError.AllocZFailed);
    machine.temp_prefix_path = temp_prefix_path;
    machine.updater.temp_prefix_path = temp_prefix_path.ptr;

    std.Io.Dir.cwd().createDirPath(machine.updater.io, temp_prefix_path) catch return machine.stateFailed(UpdateError.CheckoutFailed);

    var options = std.mem.zeroes(c_libs.OstreeRepoCheckoutAtOptions);
    options.mode = c_libs.OSTREE_REPO_CHECKOUT_MODE_NONE;
    options.overwrite_mode = c_libs.OSTREE_REPO_CHECKOUT_OVERWRITE_ADD_FILES;
    options.no_copy_fallback = 0;

    if (c_libs.ostree_repo_checkout_at(repo, &options, std.c.AT.FDCWD, temp_prefix_path.ptr, checksum, machine.updater.cancellable, &machine.updater.gerror) == 0) return machine.stateFailed(UpdateError.CheckoutFailed);

    c_libs.g_free(machine.commit_checksum);
    machine.commit_checksum = null;

    c_libs.g_object_unref(repo);
    machine.repo = null;

    return .done;
}
