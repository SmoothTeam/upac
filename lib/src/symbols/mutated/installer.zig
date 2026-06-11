// ── Imports ─────────────────────────────────────────────────────────────────────
const std = @import("std");

const types = @import("upac-types");
const Package = types.Package;
const PackageMeta = types.PackageMeta;

const ErrorCode = types.ErrorCode;
const Operation = types.Operation;

const fromError = types.fromError;

const ffi = @import("upac-ffi");
const CPackage = ffi.CPackage;
const CPackageMeta = ffi.CPackageMeta;
const CInstallRequest = ffi.CMutatedRequest;
const HookFn = ffi.HookFn;

const installer_module = @import("upac-installer");
const InstallData = installer_module.InstallData;
const InstallerMachine = installer_module.InstallerMachine;

// The main entry point for package installation. It gathers installation data from the request, initializes the installation engine, and returns an error code as an i32
pub fn install(install_request_c: CInstallRequest) callconv(.c) i32 {
    install_request_c.validate() catch |err| return @intFromEnum(fromError(err, Operation.install));

    if (install_request_c.packages) |packages_ptr| {
        const packages_slice = packages_ptr[0..install_request_c.packages_count];
        for (packages_slice) |package| {
            package.validate() catch |err| return @intFromEnum(fromError(err, Operation.install));
        }
    }

    const install_packages = collectInstallEntries(install_request_c, ffi.getAllocator()) catch |err| return @intFromEnum(fromError(err, Operation.install));
    defer ffi.getAllocator().free(install_packages);

    const install_data = InstallData{
        .packages = install_packages,
        .branch = install_request_c.branch.asZ(),
        .repo_path = install_request_c.repo_path.asZ(),
        .root_path = install_request_c.root_path.asZ(),
        .tmp_path = install_request_c.tmp_path.asZ(),
        .on_hook = install_request_c.on_hook,
        .hook_ctx = install_request_c.hook_ctx,
        .cancel_token = install_request_c.cancel_token orelse return @intFromEnum(fromError(error.InvalidEntry, Operation.install)),
    };

    InstallerMachine.run(install_data, ffi.getAllocator()) catch |err| return @intFromEnum(fromError(err, Operation.install));

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
                .version = package_meta_c.version.toVersion(),
                .arch = package_meta_c.arch.toSlice(),
                .arch_sub = if (package_meta_c.arch_sub.ptr != null) package_meta_c.arch_sub.toSlice() else null,
                .maintainer = package_meta_c.maintainer.toSlice(),
                .description = package_meta_c.description.toSlice(),
                .license = package_meta_c.license.toSlice(),
                .url = package_meta_c.url.toSlice(),
                .sha256 = package_meta_c.sha256,
            },
            .temp_package_path = package_entry_c.temp_path.asZ(),
        };
    }

    return install_packges;
}
