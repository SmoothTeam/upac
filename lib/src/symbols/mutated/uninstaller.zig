// ── Imports ─────────────────────────────────────────────────────────────────────
const std = @import("std");

const types = @import("upac-types");
const ErrorCode = types.ErrorCode;
const Operation = types.Operation;

const fromError = types.fromError;

const ffi = @import("upac-ffi");
const CSlice = ffi.CSlice;
const CUninstallPackage = ffi.CUninstallPackage;
const CUninstallRequest = ffi.CMutatedRequest;
const UninstallProgressFn = ffi.UninstallProgressFn;

const UninstallPackage = types.UninstallPackage;

const uninstaller_module = @import("upac-uninstaller");
const UninstallData = uninstaller_module.UninstallData;
const UninstallerMachine = uninstaller_module.UninstallerMachine;

// An exported function for deleting a package. It extracts the parameters (paths, package name, retry limits) and initiates the deletion process
pub fn uninstall(uninstall_request_c: CUninstallRequest) callconv(.c) i32 {
    uninstall_request_c.validate() catch |err| return @intFromEnum(fromError(err, Operation.uninstall));

    const required_fields = [_]CSlice{ uninstall_request_c.repo_path, uninstall_request_c.root_path, uninstall_request_c.branch };
    for (required_fields) |field| if (field.len == 0 or field.ptr[field.len] != 0) return @intFromEnum(fromError(error.InvalidEntry, Operation.uninstall));

    const packages_c_null = uninstall_request_c.uninstall_packages orelse return @intFromEnum(fromError(error.InvalidEntry, Operation.uninstall));
    if (uninstall_request_c.uninstall_packages_len == 0) return @intFromEnum(fromError(error.InvalidEntry, Operation.uninstall));

    const packages_c = packages_c_null[0..uninstall_request_c.uninstall_packages_len];
    for (packages_c) |pkg| pkg.validate() catch return @intFromEnum(fromError(error.InvalidEntry, Operation.uninstall));

    const packages = ffi.getAllocator().alloc(UninstallPackage, packages_c.len) catch return @intFromEnum(ErrorCode.out_of_memory);
    defer ffi.getAllocator().free(packages);

    for (packages_c, packages) |pkg_c, *pkg| pkg.* = .{
        .name = pkg_c.name.toSlice(),
        .arch = pkg_c.arch.toSlice(),
        .arch_sub = if (pkg_c.arch_sub.ptr != null) pkg_c.arch_sub.toSlice() else null,
    };

    const uninstall_data = UninstallData{
        .packages = packages,
        .branch = uninstall_request_c.branch.asZ(),
        .repo_path = uninstall_request_c.repo_path.asZ(),
        .root_path = uninstall_request_c.root_path.asZ(),
        .on_progress = if (uninstall_request_c.on_progress) |cb| @as(UninstallProgressFn, @ptrCast(cb)) else null,
        .progress_ctx = uninstall_request_c.progress_ctx,
        .cancel_token = uninstall_request_c.cancel_token orelse return @intFromEnum(fromError(error.InvalidEntry, Operation.uninstall)),
    };

    UninstallerMachine.run(uninstall_data, ffi.getAllocator()) catch |err| return @intFromEnum(fromError(err, Operation.uninstall));

    return @intFromEnum(ErrorCode.ok);
}
