const std = @import("std");

const c_libs = @import("c-libs");

const rollback = @import("../rollback.zig");
const RollbackMachine = rollback.RollbackMachine;
const RollbackError = rollback.RollbackError;

const resolveTempDir = @import("utils.zig").resolveTempDir;

// ── CheckoutState ─────────────────────────────────────────────────────────────
const CheckoutState = enum {
    open_repo,
    checkout,
    close_repo,
    done,
};

// ── CheckoutMachine ───────────────────────────────────────────────────────────
pub const CheckoutMachine = struct {
    rollback: *RollbackMachine,

    repo: ?*c_libs.OstreeRepo = null,

    temp_prefix_path: ?[:0]u8 = null,

    fn stateFailed(self: *CheckoutMachine, err: RollbackError) RollbackError {
        if (self.repo) |repo| {
            c_libs.g_object_unref(repo);
            self.repo = null;
        }

        if (self.temp_prefix_path) |path| {
            std.Io.Dir.cwd().deleteTree(self.rollback.io, path) catch {};

            self.rollback.allocator.free(path);

            self.temp_prefix_path = null;
            self.rollback.temp_prefix_path = null;
        }
        return err;
    }
};

// ── Trampoline ────────────────────────────────────────────────────────────────
pub fn run(machine: *RollbackMachine) RollbackError!void {
    var checkout_machine = CheckoutMachine{ .rollback = machine };

    var state = CheckoutState.open_repo;
    while (state != .done) {
        if (machine.cancellable) |cancellable| if (c_libs.g_cancellable_is_cancelled(cancellable) != 0) return checkout_machine.stateFailed(RollbackError.Cancelled);

        state = switch (state) {
            .open_repo => try stateOpenRepo(&checkout_machine),
            .checkout => try stateCheckout(&checkout_machine),
            .close_repo => stateCloseRepo(&checkout_machine),
            .done => unreachable,
        };
    }
}

// ── States ────────────────────────────────────────────────────────────────────
fn stateOpenRepo(machine: *CheckoutMachine) RollbackError!CheckoutState {
    const gfile = c_libs.g_file_new_for_path(machine.rollback.data.repo_path);
    defer c_libs.g_object_unref(gfile);

    const repo = c_libs.ostree_repo_new(gfile);
    if (c_libs.ostree_repo_open(repo, machine.rollback.cancellable, &machine.rollback.gerror) == 0) {
        c_libs.g_object_unref(repo);
        return machine.stateFailed(RollbackError.RepoOpenFailed);
    }
    machine.repo = repo;

    return .checkout;
}

fn stateCheckout(machine: *CheckoutMachine) RollbackError!CheckoutState {
    const repo = machine.repo orelse return machine.stateFailed(RollbackError.RepoOpenFailed);

    const root_path = std.mem.span(machine.rollback.data.root_path);

    const temp_prefix_path = resolveTempDir(root_path, machine.rollback.allocator, machine.rollback.io) catch return machine.stateFailed(RollbackError.StagingFailed);
    machine.temp_prefix_path = temp_prefix_path;
    machine.rollback.temp_prefix_path = temp_prefix_path.ptr;

    std.Io.Dir.cwd().createDirPath(machine.rollback.io, temp_prefix_path) catch return machine.stateFailed(RollbackError.StagingFailed);

    var options = std.mem.zeroes(c_libs.OstreeRepoCheckoutAtOptions);
    options.mode = c_libs.OSTREE_REPO_CHECKOUT_MODE_NONE;
    options.overwrite_mode = c_libs.OSTREE_REPO_CHECKOUT_OVERWRITE_ADD_FILES;
    options.no_copy_fallback = 0;

    if (c_libs.ostree_repo_checkout_at(repo, &options, std.c.AT.FDCWD, temp_prefix_path.ptr, machine.rollback.data.commit_hash, machine.rollback.cancellable, &machine.rollback.gerror) == 0) return machine.stateFailed(RollbackError.RollbackFailed);

    return .close_repo;
}

fn stateCloseRepo(machine: *CheckoutMachine) CheckoutState {
    if (machine.repo) |repo| {
        c_libs.g_object_unref(repo);
        machine.repo = null;
    }
    return .done;
}
