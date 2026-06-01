const std = @import("std");

const c_libs = @import("c-libs");

const types = @import("upac-types");
const PREFIX = types.paths.prefix;

const files = @import("../files.zig");
const FilesMachine = files.FilesMachine;
const FilesError = files.FilesError;

// ── CheckoutState ─────────────────────────────────────────────────────────────
const CheckoutState = enum {
    open_repo,
    resolve_commit,
    create_temp_dir,
    checkout,
    close_repo,
    done,
};

// ── CheckoutMachine ───────────────────────────────────────────────────────────
const CheckoutMachine = struct {
    files: *FilesMachine,

    repo: ?*c_libs.OstreeRepo = null,
    commit_checksum: [*c]u8 = null,

    fn stateFailed(self: *CheckoutMachine, err: FilesError) FilesError {
        if (self.commit_checksum != null) {
            c_libs.g_free(self.commit_checksum);
            self.commit_checksum = null;
        }

        if (self.repo) |repo| {
            c_libs.g_object_unref(repo);
            self.repo = null;
        }

        if (self.files.temp_prefix_path) |path| {
            const path_slice = std.mem.span(path);
            std.Io.Dir.cwd().deleteTree(self.files.io, path_slice) catch {};
            self.files.allocator.free(path_slice);
            self.files.temp_prefix_path = null;
        }

        return err;
    }
};

// ── Trampoline ────────────────────────────────────────────────────────────────
pub fn run(machine: *FilesMachine) FilesError!void {
    var checkout_machine = CheckoutMachine{ .files = machine };

    var state = CheckoutState.open_repo;
    while (state != .done) {
        if (machine.cancellable) |cancellable| if (c_libs.g_cancellable_is_cancelled(cancellable) != 0) return checkout_machine.stateFailed(FilesError.Cancelled);

        state = switch (state) {
            .open_repo => try stateOpenRepo(&checkout_machine),
            .resolve_commit => try stateResolveCommit(&checkout_machine),
            .create_temp_dir => try stateCreateTempDir(&checkout_machine),
            .checkout => try stateCheckout(&checkout_machine),
            .close_repo => stateCloseRepo(&checkout_machine),
            .done => unreachable,
        };
    }
}

// ── States ────────────────────────────────────────────────────────────────────
fn stateOpenRepo(machine: *CheckoutMachine) FilesError!CheckoutState {
    const gfile = c_libs.g_file_new_for_path(machine.files.data.repo_path);
    defer c_libs.g_object_unref(gfile);

    const repo = c_libs.ostree_repo_new(gfile);
    if (c_libs.ostree_repo_open(repo, machine.files.cancellable, &machine.files.gerror) == 0) {
        c_libs.g_object_unref(repo);
        return machine.stateFailed(FilesError.RepoOpenFailed);
    }
    machine.repo = repo;

    return .resolve_commit;
}

fn stateResolveCommit(machine: *CheckoutMachine) FilesError!CheckoutState {
    const repo = machine.repo orelse return machine.stateFailed(FilesError.RepoOpenFailed);

    if (c_libs.ostree_repo_resolve_rev(repo, machine.files.data.branch, 0, &machine.commit_checksum, &machine.files.gerror) == 0) return machine.stateFailed(FilesError.CheckoutFailed);
    if (machine.commit_checksum == null) return machine.stateFailed(FilesError.CheckoutFailed);

    return .create_temp_dir;
}

fn stateCreateTempDir(machine: *CheckoutMachine) FilesError!CheckoutState {
    var name_buf: [128]u8 = undefined;
    var timespec: std.os.linux.timespec = undefined;
    _ = std.os.linux.clock_gettime(std.os.linux.CLOCK.REALTIME, &timespec);
    const timestamp: i64 = @as(i64, timespec.sec) * 1000 + @divTrunc(@as(i64, timespec.nsec), 1_000_000);

    const root_path = std.mem.span(machine.files.data.root_path);

    const temp_name = std.fmt.bufPrint(&name_buf, "{s}-files-{d}", .{ PREFIX, timestamp }) catch return machine.stateFailed(FilesError.AllocFailed);

    const temp_prefix_path = std.fs.path.joinZ(machine.files.allocator, &.{ root_path, temp_name }) catch return machine.stateFailed(FilesError.AllocFailed);
    machine.files.temp_prefix_path = temp_prefix_path;

    std.Io.Dir.cwd().createDirPath(machine.files.io, temp_prefix_path) catch return machine.stateFailed(FilesError.CheckoutFailed);

    return .checkout;
}

fn stateCheckout(machine: *CheckoutMachine) FilesError!CheckoutState {
    const repo = machine.repo orelse return machine.stateFailed(FilesError.RepoOpenFailed);
    const checksum = machine.commit_checksum orelse return machine.stateFailed(FilesError.CheckoutFailed);
    const temp_prefix_path = machine.files.temp_prefix_path orelse return machine.stateFailed(FilesError.CheckoutFailed);

    var options = std.mem.zeroes(c_libs.OstreeRepoCheckoutAtOptions);
    options.mode = c_libs.OSTREE_REPO_CHECKOUT_MODE_NONE;
    options.overwrite_mode = c_libs.OSTREE_REPO_CHECKOUT_OVERWRITE_ADD_FILES;
    options.no_copy_fallback = 0;

    if (c_libs.ostree_repo_checkout_at(repo, &options, std.c.AT.FDCWD, temp_prefix_path, checksum, machine.files.cancellable, &machine.files.gerror) == 0) return machine.stateFailed(FilesError.CheckoutFailed);

    c_libs.g_free(machine.commit_checksum);
    machine.commit_checksum = null;

    return .close_repo;
}

fn stateCloseRepo(machine: *CheckoutMachine) CheckoutState {
    if (machine.repo) |repo| {
        c_libs.g_object_unref(repo);
        machine.repo = null;
    }

    return .done;
}
