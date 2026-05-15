// ── Imports ─────────────────────────────────────────────────────────────────────
const std = @import("std");

const uninstaller = @import("../uninstaller.zig");
const c_libs = uninstaller.ffi.c_libs;

const UninstallerMachine = uninstaller.UninstallerMachine;
const UninstallerError = uninstaller.UninstallerError;

const VerifyingMachine = @import("verifying.zig").VerifyingMachine;

pub fn dirSize(machine: *UninstallerMachine, root_path: []const u8) !usize {
    var total_size: usize = 0;

    var dir = std.Io.Dir.openDirAbsolute(machine.io, root_path, .{ .iterate = true }) catch return 0;
    defer dir.close(machine.io);

    var walker = try dir.walk(machine.allocator);
    defer walker.deinit();

    while (try walker.next(machine.io)) |entry| {
        if (entry.kind != .file) continue;
        const file_stats = entry.dir.statFile(machine.io, entry.basename, .{}) catch continue;
        total_size += file_stats.size;
    }

    return total_size;
}

pub fn loadCommitBody(machine: *VerifyingMachine, checksum: [*c]const u8) UninstallerError![]const u8 {
    var commit_variant: ?*c_libs.GVariant = null;
    defer if (commit_variant) |variant| c_libs.g_variant_unref(variant);

    const repo = machine.repo orelse return UninstallerError.RepoOpenFailed;
    if (c_libs.ostree_repo_load_variant(repo, c_libs.OSTREE_OBJECT_TYPE_COMMIT, checksum, &commit_variant, &machine.uninstaller.gerror) == 0) return UninstallerError.CommitNotFound;

    var body_len: usize = 0;
    const commit_body_variant = c_libs.g_variant_get_child_value(commit_variant, 4);
    defer if (commit_body_variant) |variant| c_libs.g_variant_unref(variant);

    const body_ptr = c_libs.g_variant_get_string(commit_body_variant, &body_len);
    return machine.uninstaller.allocator.dupe(u8, body_ptr[0..body_len]) catch return UninstallerError.AllocZFailed;
}
