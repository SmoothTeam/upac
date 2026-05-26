const std = @import("std");

const c_libs = @import("c-libs");

const types = @import("upac-types");
const FileRecord = types.FileRecord;

const database = @import("upac-database");
const Database = database.Database;
const package_list = database.packages.list;
const files_list = database.files.list;
const exists = database.packages.exists;

const diff_module = @import("../diff.zig");
const DiffError = diff_module.DiffError;

const check = diff_module.DiffMachine.check;

const PreparationMachine = @import("preparation.zig").PreparationMachine;

pub fn checkoutDb(machine: *PreparationMachine, checksum: [*c]const u8) DiffError!void {
    const destination_path = machine.current_database_path orelse return DiffError.CheckoutFailed;

    const destination_pathz = machine.diff.allocator.dupeZ(u8, destination_path) catch return DiffError.AllocFailed;
    defer machine.diff.allocator.free(destination_pathz);

    const subpath = std.fs.path.joinZ(machine.diff.allocator, &.{ types.paths.prefix, types.paths.db_path }) catch return DiffError.AllocFailed;
    defer machine.diff.allocator.free(subpath);

    var checkout_options = std.mem.zeroes(c_libs.OstreeRepoCheckoutAtOptions);
    checkout_options.subpath = subpath;

    try check(c_libs.ostree_repo_checkout_at(machine.repo orelse return DiffError.CheckoutFailed, &checkout_options, c_libs.AT_FDCWD, destination_pathz, checksum, machine.diff.cancellable, &machine.diff.gerror), .CheckoutFailed);
}

pub fn buildFilePkgMap(base: Database, allocator: std.mem.Allocator, out: *std.StringHashMap(FileRecord)) DiffError!void {
    const package_metas = package_list(base) catch return DiffError.ReadDatabaseFailed;
    defer {
        for (package_metas) |*meta| meta.deinit(allocator);
        allocator.free(package_metas);
    }

    for (package_metas) |meta| {
        const uuid = (exists(base, meta.name, meta.arch, meta.arch_sub) catch continue) orelse continue;

        const file_entries = files_list(base, uuid) catch continue;
        defer {
            for (file_entries) |*file_entry| file_entry.deinit(allocator);
            allocator.free(file_entries);
        }

        for (file_entries) |file_entry| {
            if (out.contains(file_entry.path)) continue;

            const path_dupe = allocator.dupe(u8, file_entry.path) catch return DiffError.AllocFailed;
            errdefer allocator.free(path_dupe);

            const pkg_name_dupe = allocator.dupe(u8, meta.name) catch return DiffError.AllocFailed;
            errdefer allocator.free(pkg_name_dupe);

            out.put(path_dupe, .{
                .sha256 = file_entry.sha256,
                .is_user = file_entry.is_user,
                .pkg_name = pkg_name_dupe,
            }) catch return DiffError.AllocFailed;
        }
    }
}
