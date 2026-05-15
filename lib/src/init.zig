// ── Imports ─────────────────────────────────────────────────────────────────────
const posix = std.posix;

const constants = @import("upac-constants");
const PREFIX = constants.PREFIX;

pub const ffi = @import("upac-ffi");
const c_libs = ffi.c_libs;

const CRepoMode = ffi.CRepoMode;

const CancelToken = ffi.CancelToken;
const cancelGCancellable = ffi.cancelGCancellable;

// ── Public imports ─────────────────────────────────────────────────────────────────────
pub const std = @import("std");

pub const InitError = error{
    AlreadyInitialized,
    RootNotFound,
    PrefixNotFound,
    NotADirectory,
    CreateDirFailed,
    OstreeInitFailed,
    DirectoryNotEmpty,
    SymlinkFailed,
};

// ── Public API ─────────────────────────────────────────────────────────────
pub fn initSystem(repo_path_c: [*:0]const u8, root_path_c: [*:0]const u8, repo_mode: CRepoMode, branch_c: [*:0]const u8, symlinks: []const []const u8, cancel_token: *CancelToken, allocator: std.mem.Allocator) !void {
    var gerror: ?*c_libs.GError = null;
    defer if (gerror) |err| c_libs.g_error_free(err);

    const cancellable = c_libs.g_cancellable_new() orelse return error.OutOfMemory;
    defer c_libs.g_object_unref(cancellable);

    cancel_token.hook = cancelGCancellable;
    cancel_token.hook_ctx = cancellable;
    defer cancel_token.reset();

    if (c_libs.g_cancellable_is_cancelled(cancellable) != 0) return error.Cancelled;
    if (!try checkDirExists(root_path_c)) return InitError.RootNotFound;

    if (c_libs.g_cancellable_is_cancelled(cancellable) != 0) return error.Cancelled;
    const io = std.Io.Threaded.global_single_threaded.io();

    const prefix_path_c = std.fs.path.joinZ(allocator, &.{ std.mem.span(root_path_c), PREFIX }) catch return InitError.PrefixNotFound;
    defer allocator.free(prefix_path_c);
    if (try checkFileExists(prefix_path_c)) return InitError.PrefixNotFound;
    if (!try checkDirExists(prefix_path_c)) {
        std.Io.Dir.createDirAbsolute(io, prefix_path_c, .default_dir) catch return InitError.CreateDirFailed;
    }

    for (symlinks) |symlink_name| {
        if (c_libs.g_cancellable_is_cancelled(cancellable) != 0) return error.Cancelled;

        // Ensure <root>/<PREFIX>/<name> directory exists (symlink target)
        const target_dir_path = std.fs.path.joinZ(allocator, &.{ std.mem.span(root_path_c), PREFIX, symlink_name }) catch return error.OutOfMemory;
        defer allocator.free(target_dir_path);
        if (!try checkDirExists(target_dir_path)) {
            std.Io.Dir.createDirAbsolute(io, target_dir_path, .default_dir) catch return InitError.CreateDirFailed;
        }

        // Relative symlink target: <PREFIX>/<name>
        const link_target = std.fs.path.joinZ(allocator, &.{ PREFIX, symlink_name }) catch return error.OutOfMemory;
        defer allocator.free(link_target);

        // Absolute path of the symlink itself: <root>/<name>
        const link_path = std.fs.path.joinZ(allocator, &.{ std.mem.span(root_path_c), symlink_name }) catch return error.OutOfMemory;
        defer allocator.free(link_path);

        var readlink_buf: [std.fs.max_path_bytes]u8 = undefined;
        const existing_len = std.Io.Dir.readLinkAbsolute(io, link_path, &readlink_buf) catch |err| switch (err) {
            error.FileNotFound => {
                // target is relative (e.g. "usr/opt"), link_path is absolute
                std.Io.Dir.cwd().symLink(io, link_target, link_path, .{}) catch return InitError.SymlinkFailed;
                continue;
            },
            // Not a symlink (EINVAL), or any other error — something is in the way
            else => return InitError.SymlinkFailed,
        };

        // Symlink exists: accept only if it already points to the right target
        if (!std.mem.eql(u8, readlink_buf[0..existing_len], link_target)) return InitError.SymlinkFailed;
    }

    if (c_libs.g_cancellable_is_cancelled(cancellable) != 0) return error.Cancelled;
    if (try checkFileExists(repo_path_c)) return InitError.NotADirectory;

    if (c_libs.g_cancellable_is_cancelled(cancellable) != 0) return error.Cancelled;
    if (!try checkDirExists(repo_path_c)) {
        std.Io.Dir.createDirAbsolute(io, std.mem.span(repo_path_c), .default_dir) catch return InitError.CreateDirFailed;
    } else {
        var dir = try std.Io.Dir.openDirAbsolute(io, std.mem.span(repo_path_c), .{ .iterate = true });
        defer dir.close(io);

        var iterator = dir.iterate();
        var is_empty = true;
        while (try iterator.next(io)) |entry| {
            is_empty = false;
            if (std.mem.eql(u8, entry.name, "config")) return InitError.AlreadyInitialized;
        }
        if (!is_empty) return InitError.DirectoryNotEmpty;
    }

    if (c_libs.g_cancellable_is_cancelled(cancellable) != 0) return error.Cancelled;
    try initOstreeRepo(repo_path_c, repo_mode, branch_c, cancellable, &gerror);
}

fn checkDirExists(path: [*:0]const u8) !bool {
    const io = std.Io.Threaded.global_single_threaded.io();
    const stat = std.Io.Dir.cwd().statFile(io, std.mem.span(path), .{}) catch |err| switch (err) {
        error.FileNotFound => return false,
        else => return err,
    };
    return stat.kind == .directory;
}

fn checkFileExists(path: [*:0]const u8) !bool {
    const io = std.Io.Threaded.global_single_threaded.io();
    const stat = std.Io.Dir.cwd().statFile(io, std.mem.span(path), .{}) catch |err| switch (err) {
        error.FileNotFound => return false,
        else => return err,
    };
    return stat.kind == .file;
}

fn initOstreeRepo(repo_path_c: [*:0]const u8, repo_mode: CRepoMode, branch_c: [*:0]const u8, cancellable: *c_libs.GCancellable, gerror: *?*c_libs.GError) !void {
    const struct_g_file = c_libs.g_file_new_for_path(repo_path_c);
    defer c_libs.g_object_unref(struct_g_file);

    const struct_ostree_repo = c_libs.ostree_repo_new(struct_g_file);
    defer c_libs.g_object_unref(struct_ostree_repo);

    const ostree_mode: c_libs.OstreeRepoMode = switch (repo_mode) {
        .archive => c_libs.OSTREE_REPO_MODE_ARCHIVE,
        .bare => c_libs.OSTREE_REPO_MODE_BARE,
        .bare_user => c_libs.OSTREE_REPO_MODE_BARE_USER,
    };

    if (c_libs.ostree_repo_create(struct_ostree_repo, ostree_mode, cancellable, gerror) == 0) return InitError.OstreeInitFailed;

    if (c_libs.ostree_repo_prepare_transaction(struct_ostree_repo, null, cancellable, gerror) == 0) return InitError.OstreeInitFailed;

    c_libs.ostree_repo_transaction_set_ref(struct_ostree_repo, null, branch_c, null);

    if (c_libs.ostree_repo_commit_transaction(struct_ostree_repo, null, cancellable, gerror) == 0) {
        _ = c_libs.ostree_repo_abort_transaction(struct_ostree_repo, cancellable, null);
        return InitError.OstreeInitFailed;
    }
}
