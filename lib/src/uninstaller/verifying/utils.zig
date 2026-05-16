// ── Imports ─────────────────────────────────────────────────────────────────────
const std = @import("std");

const c_libs = @import("c-libs");

const uninstaller = @import("../uninstaller.zig");

const UninstallerMachine = uninstaller.UninstallerMachine;
const UninstallerError = uninstaller.UninstallerError;

const VerifyingMachine = @import("verifying.zig").VerifyingMachine;

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
