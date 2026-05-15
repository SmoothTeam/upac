// ── Imports ─────────────────────────────────────────────────────────────────────
const std = @import("std");

const types = @import("upac-types");
const Package = types.Package;
const PackageMeta = types.PackageMeta;

const ErrorCode = types.ErrorCode;
const Operation = types.Operation;

const fromError = types.fromError;

const ffi = @import("upac-ffi");
const CSlice = ffi.CSlice;
const CPackageMeta = ffi.CPackageMeta;
const CInstallRequest = ffi.CMutatedRequest;
const InstallProgressFn = ffi.InstallProgressFn;

const InstallProgressEvent = ffi.InstallProgressEvent;

const installer_module = @import("upac-installer");

// The main entry point for package installation. It gathers installation data from the request, initializes the installation engine, and returns an error code as an i32
pub fn install(install_request_c: CInstallRequest) callconv(.c) i32 {
    install_request_c.validate() catch |err| return @intFromEnum(fromError(err, Operation.install));

    const install_packages = collectInstallEntries(install_request_c, ffi.allocator()) catch |err| return @intFromEnum(fromError(err, Operation.install));
    defer ffi.allocator().free(install_packages);

    const install_data = installer_module.InstallData{
        .packages = install_packages,
        .branch = install_request_c.branch.asZ(),
        .repo_path = install_request_c.repo_path.asZ(),
        .root_path = install_request_c.root_path.asZ(),
        .on_progress = if (install_request_c.on_progress) |cb| @as(InstallProgressFn, @ptrCast(cb)) else null,
        .progress_ctx = install_request_c.progress_ctx,
        .cancel_token = install_request_c.cancel_token orelse return @intFromEnum(fromError(error.InvalidEntry, Operation.install)),
    };

    installer_module.InstallerMachine.run(install_data, ffi.allocator()) catch |err| return @intFromEnum(fromError(err, Operation.install));

    return @intFromEnum(ErrorCode.ok);
}

fn collectInstallEntries(install_request_c: CInstallRequest, allocator: std.mem.Allocator) ![]const Package {
    if (install_request_c.packages_count > 0 and install_request_c.packages == null) return error.InvalidEntry;

    const packages_entry_c_ptr = install_request_c.packages orelse return error.InvalidEntry;

    const packages_entrys_c = packages_entry_c_ptr[0..install_request_c.packages_count];

    const install_packges = allocator.alloc(Package, packages_entrys_c.len) catch return error.OutOfMemory;
    errdefer allocator.free(install_packges);

    for (packages_entrys_c, 0..) |package_entry_c, index| {
        const package_meta_c: *CPackageMeta = @ptrCast(@alignCast(package_entry_c.meta));

        install_packges[index] = .{
            .meta = .{
                .name = package_meta_c.name.toSlice(),
                .version = package_meta_c.version.toSlice(),
                .size = @intCast(package_meta_c.size),
                .architecture = package_meta_c.architecture.toSlice(),
                .author = package_meta_c.author.toSlice(),
                .description = package_meta_c.description.toSlice(),
                .license = package_meta_c.license.toSlice(),
                .url = package_meta_c.url.toSlice(),
                .packager = package_meta_c.packager.toSlice(),
                .installed_at = package_meta_c.installed_at,
                .checksum = package_meta_c.checksum.toSlice(),
            },
            .path = package_entry_c.temp_path.asZ(),
            .checksum = package_entry_c.checksum.toSlice(),
        };
    }

    return install_packges;
}
