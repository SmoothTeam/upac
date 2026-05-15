// ── Imports ─────────────────────────────────────────────────────────────────────
pub const std = @import("std");

const list = @import("list.zig");
const c_libs = list.ffi.c_libs;

const CSlice = list.ffi.CSlice;
const CPackageMeta = list.ffi.CPackageMeta;
const CCommitEntry = list.ffi.CCommitEntry;

const DB_RELATIVE_PATH = list.types.DB_RELATIVE_PATH;

const readMeta = list.database.readMeta;

const ListMachine = list.ListMachine;
const ListError = list.ListError;

const utils = @import("utils.zig");
const getRefBody = utils.getRefBody;
const parsePackageBody = utils.parsePackageBody;
const freeStringMap = utils.freeStringMap;

pub fn stateOpenRepo(machine: *ListMachine) ListError!void {
    const gfile = c_libs.g_file_new_for_path(machine.data.repo_path);
    defer c_libs.g_object_unref(gfile);

    const repo = c_libs.ostree_repo_new(gfile);
    if (c_libs.ostree_repo_open(repo, machine.cancellable, &machine.gerror) == 0) {
        c_libs.g_object_unref(repo);
        return stateFailed(ListError.RepoOpenFailed);
    }
    machine.repo = repo;
}

pub fn stateListPackages(machine: *ListMachine) ListError![]CPackageMeta {
    var package_meta_list_c = std.ArrayList(CPackageMeta).empty;
    errdefer {
        for (package_meta_list_c.items) |package| {
            machine.allocator.free(package.name.toSlice());
            machine.allocator.free(package.version.toSlice());
            machine.allocator.free(package.architecture.toSlice());
            machine.allocator.free(package.description.toSlice());
            machine.allocator.free(package.license.toSlice());
            machine.allocator.free(package.packager.toSlice());
            machine.allocator.free(package.author.toSlice());
            machine.allocator.free(package.checksum.toSlice());
            machine.allocator.free(package.url.toSlice());
        }
        package_meta_list_c.deinit(machine.allocator);
    }
    const root_path = std.mem.span(machine.data.root_path);

    const database_path = std.fs.path.join(machine.allocator, &.{ root_path, DB_RELATIVE_PATH }) catch return stateFailed(ListError.AllocFailed);
    defer machine.allocator.free(database_path);

    const commit_body = getRefBody(machine) catch |err| return stateFailed(err);
    defer if (commit_body) |body| machine.allocator.free(body);

    const unwraped_commit_body = commit_body orelse return package_meta_list_c.toOwnedSlice(machine.allocator) catch return stateFailed(ListError.AllocFailed);

    var packages_map = parsePackageBody(unwraped_commit_body, machine.allocator) catch |err| return stateFailed(err);
    defer freeStringMap(&packages_map, machine.allocator);

    var packages_map_iter = packages_map.iterator();
    while (packages_map_iter.next()) |package| {
        const package_meta = readMeta(database_path, package.value_ptr.*, machine.allocator) catch continue;
        package_meta_list_c.append(machine.allocator, .{
            .name = CSlice.fromSlice(package_meta.name),
            .version = CSlice.fromSlice(package_meta.version),
            .architecture = CSlice.fromSlice(package_meta.architecture),
            .author = CSlice.fromSlice(package_meta.author),
            .description = CSlice.fromSlice(package_meta.description),
            .license = CSlice.fromSlice(package_meta.license),
            .url = CSlice.fromSlice(package_meta.url),
            .packager = CSlice.fromSlice(package_meta.packager),
            .checksum = CSlice.fromSlice(package_meta.checksum),
            .size = @intCast(package_meta.size),
            .installed_at = package_meta.installed_at,
        }) catch |err| return stateFailed(err);
    }

    return package_meta_list_c.toOwnedSlice(machine.allocator) catch |err| return stateFailed(err);
}

pub fn stateListCommits(machine: *ListMachine) ListError![]CCommitEntry {
    var commits_list_c = std.ArrayList(CCommitEntry).empty;
    errdefer {
        for (commits_list_c.items) |commit_array| {
            machine.allocator.free(commit_array.checksum.toSlice());
            machine.allocator.free(commit_array.subject.toSlice());
        }
        commits_list_c.deinit(machine.allocator);
    }

    var current_checksum: [*c]u8 = null;
    defer if (current_checksum != null) c_libs.g_free(current_checksum);

    var is_first = true;
    var checksum = current_checksum;

    const repo = machine.repo orelse return ListError.RepoOpenFailed;

    if (c_libs.ostree_repo_resolve_rev(repo, machine.data.branch, 1, &current_checksum, &machine.gerror) == 0) return commits_list_c.toOwnedSlice(machine.allocator) catch return stateFailed(ListError.AllocFailed);

    while (checksum != null) {
        if (machine.cancellable) |cancellable| if (c_libs.g_cancellable_is_cancelled(cancellable) != 0) return stateFailed(ListError.Cancelled);

        var commit_variant: ?*c_libs.GVariant = null;
        if (c_libs.ostree_repo_load_variant(repo, c_libs.OSTREE_OBJECT_TYPE_COMMIT, checksum, &commit_variant, &machine.gerror) == 0) {
            if (!is_first) c_libs.g_free(checksum);
            break;
        }
        defer if (commit_variant) |variant| c_libs.g_variant_unref(variant);

        const subject_variant = c_libs.g_variant_get_child_value(commit_variant, 3);
        defer if (subject_variant) |variant| c_libs.g_variant_unref(variant);

        var subject_len: usize = 0;
        const subject_ptr = c_libs.g_variant_get_string(subject_variant, &subject_len);

        const checksum_dupe = machine.allocator.dupe(u8, std.mem.span(checksum)) catch return stateFailed(ListError.AllocFailed);
        const subject_dupe = machine.allocator.dupe(u8, subject_ptr[0..subject_len]) catch return stateFailed(ListError.AllocFailed);

        commits_list_c.append(machine.allocator, .{
            .checksum = CSlice.fromSlice(checksum_dupe),
            .subject = CSlice.fromSlice(subject_dupe),
        }) catch |err| return stateFailed(err);

        const parent = c_libs.ostree_commit_get_parent(commit_variant);
        if (!is_first) c_libs.g_free(checksum);
        is_first = false;
        checksum = parent;
    }

    return commits_list_c.toOwnedSlice(machine.allocator) catch |err| return stateFailed(err);
}

pub fn stateFailed(err: ListError) ListError {
    return err;
}
