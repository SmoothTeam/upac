const std = @import("std");

const installer = @import("../installer.zig");
const c_libs = installer.ffi.c_libs;

const InstallerMachine = installer.InstallerMachine;
const InstallerError = installer.InstallerError;

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
    uninstaller: *InstallerMachine,

    repo: ?*c_libs.OstreeRepo = null,

    commit_checksum: [*c]u8 = null,

    temp_prefix_path: ?[:0]u8 = null,

    fn stateFailed(self: *CheckoutMachine, err: InstallerError) InstallerError {
        if (self.repo) |repo| {
            c_libs.g_object_unref(repo);
            self.repo = null;
        }
        if (self.commit_checksum != null) {
            c_libs.g_free(self.commit_checksum);
            self.commit_checksum = null;
        }
        if (self.temp_prefix_path) |path| {
            std.Io.Dir.cwd().deleteTree(self.uninstaller.io, path) catch {};
            self.uninstaller.allocator.free(path);
            self.temp_prefix_path = null;
            self.uninstaller.temp_prefix_path = null;
        }
        return err;
    }
};

// ── Trampoline ────────────────────────────────────────────────────────────────
pub fn run(machine: *InstallerMachine) InstallerError!void {
    var checkout_machine = CheckoutMachine{ .uninstaller = machine };

    if (machine.cancellable) |cancellable| if (c_libs.g_cancellable_is_cancelled(cancellable) != 0) return checkout_machine.stateFailed(InstallerError.Cancelled);

    var state = CheckoutState.open_repo;
    while (state != .done) {
        state = switch (state) {
            .open_repo => try stateOpenRepo(&checkout_machine),
            .resolve_commit => try stateResolveCommit(&checkout_machine),
            .checkout => try stateCheckout(&checkout_machine),
            .done => unreachable,
        };
    }
}

// ── States ────────────────────────────────────────────────────────────────────
fn stateOpenRepo(machine: *CheckoutMachine) InstallerError!CheckoutState {
    const gfile = c_libs.g_file_new_for_path(machine.uninstaller.data.repo_path);
    defer c_libs.g_object_unref(gfile);

    const repo = c_libs.ostree_repo_new(gfile);
    if (c_libs.ostree_repo_open(repo, machine.uninstaller.cancellable, &machine.uninstaller.gerror) == 0) {
        c_libs.g_object_unref(repo);
        return machine.stateFailed(InstallerError.RepoOpenFailed);
    }
    machine.repo = repo;

    return .resolve_commit;
}

fn stateResolveCommit(machine: *CheckoutMachine) InstallerError!CheckoutState {
    const repo = machine.repo orelse return machine.stateFailed(InstallerError.RepoOpenFailed);

    if (c_libs.ostree_repo_resolve_rev(repo, machine.uninstaller.data.branch, 0, &machine.commit_checksum, &machine.uninstaller.gerror) == 0) return machine.stateFailed(InstallerError.CommitNotFound);
    if (machine.commit_checksum == null) return machine.stateFailed(InstallerError.CommitNotFound);

    return .checkout;
}

fn stateCheckout(machine: *CheckoutMachine) InstallerError!CheckoutState {
    const repo = machine.repo orelse return machine.stateFailed(InstallerError.RepoOpenFailed);
    const checksum = machine.commit_checksum orelse return machine.stateFailed(InstallerError.CommitNotFound);

    const root_path = std.mem.span(machine.uninstaller.data.root_path);

    const temp_prefix_path = resolveTempDir(root_path, machine.uninstaller.allocator) catch return machine.stateFailed(InstallerError.AllocZFailed);
    machine.temp_prefix_path = temp_prefix_path;
    machine.uninstaller.temp_prefix_path = temp_prefix_path.ptr;

    std.Io.Dir.cwd().createDirPath(machine.uninstaller.io, temp_prefix_path) catch return machine.stateFailed(InstallerError.CheckoutFailed);

    var options = std.mem.zeroes(c_libs.OstreeRepoCheckoutAtOptions);
    options.mode = c_libs.OSTREE_REPO_CHECKOUT_MODE_NONE;
    options.overwrite_mode = c_libs.OSTREE_REPO_CHECKOUT_OVERWRITE_ADD_FILES;
    options.no_copy_fallback = 0;

    if (c_libs.ostree_repo_checkout_at(repo, &options, std.c.AT.FDCWD, temp_prefix_path.ptr, checksum, machine.uninstaller.cancellable, &machine.uninstaller.gerror) == 0) return machine.stateFailed(InstallerError.CheckoutFailed);

    c_libs.g_free(machine.commit_checksum);
    machine.commit_checksum = null;

    c_libs.g_object_unref(repo);
    machine.repo = null;

    return .done;
}
