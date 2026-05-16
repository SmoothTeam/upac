const c_libs = @import("c-libs");

const InstallerError = @import("../installer.zig").InstallerError;

const TransactionMachine = @import("transaction.zig").TransactionMachine;

pub fn loadCommitBody(machine: *TransactionMachine, checksum: [*c]const u8) InstallerError![]const u8 {
    var commit_variant: ?*c_libs.GVariant = null;
    defer if (commit_variant) |variant| c_libs.g_variant_unref(variant);

    const repo = machine.repo orelse return InstallerError.RepoOpenFailed;

    if (c_libs.ostree_repo_load_variant(repo, c_libs.OSTREE_OBJECT_TYPE_COMMIT, checksum, &commit_variant, &machine.installer.gerror) == 0) {
        return InstallerError.CommitNotFound;
    }

    var body_len: usize = 0;
    const body_variant = c_libs.g_variant_get_child_value(commit_variant, 4);
    defer if (body_variant) |variant| c_libs.g_variant_unref(variant);

    const body_ptr = c_libs.g_variant_get_string(body_variant, &body_len);
    if (body_len == 0) return InstallerError.CommitNotFound;
    return machine.installer.allocator.dupe(u8, body_ptr[0..body_len]) catch InstallerError.AllocZFailed;
}
