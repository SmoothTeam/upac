const std = @import("std");

const c_libs = @import("c-libs");

const types = @import("upac-types");
const PackageMeta = types.PackageMeta;
const DiffError = types.DiffError;

const database = @import("upac-database");
const Database = database.Database;
const package_list = database.packages.list;

const PreparationMachine = @import("preparation.zig").PreparationMachine;

pub fn checkoutDb(machine: *PreparationMachine, checksum: [*c]const u8) DiffError!void {
    const destination_path = machine.current_database_path orelse return DiffError.CheckoutFailed;

    const destination_path_c = machine.diff.allocator.dupeZ(u8, destination_path) catch return DiffError.AllocFailed;
    defer machine.diff.allocator.free(destination_path_c);

    const subpath = std.fs.path.joinZ(machine.diff.allocator, &.{ types.paths.prefix, types.paths.db_path }) catch return DiffError.AllocFailed;
    defer machine.diff.allocator.free(subpath);

    var checkout_options = std.mem.zeroes(c_libs.OstreeRepoCheckoutAtOptions);
    checkout_options.subpath = subpath;

    try machine.diff.check(c_libs.ostree_repo_checkout_at(machine.repo orelse return DiffError.CheckoutFailed, &checkout_options, c_libs.AT_FDCWD, destination_path_c, checksum, machine.diff.cancellable, &machine.diff.gerror), error.CheckoutFailed);
}

pub fn buildPkgList(base: Database, allocator: std.mem.Allocator, out: *std.ArrayList(PackageMeta)) DiffError!void {
    const package_metas = package_list(base) catch return DiffError.ReadDatabaseFailed;
    errdefer {
        for (package_metas) |*meta| meta.deinit(allocator);
        allocator.free(package_metas);
    }

    out.appendSlice(allocator, package_metas) catch return DiffError.AllocFailed;
    allocator.free(package_metas);
}
