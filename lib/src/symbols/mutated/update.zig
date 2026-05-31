// ── Imports ─────────────────────────────────────────────────────────────────────
const std = @import("std");

const types = @import("upac-types");
const Package = types.Package;
const PackageMeta = types.PackageMeta;

const ErrorCode = types.ErrorCode;
const Operation = types.Operation;

const fromError = types.fromError;

const ffi = @import("upac-ffi");
const CPackageMeta = ffi.CPackageMeta;
const CUpdateRequest = ffi.CMutatedRequest;
const UpdateProgressFn = ffi.UpdateProgressFn;

const update_module = @import("upac-update");
const UpdateData = update_module.UpdateData;
const UpdateMachine = update_module.UpdateMachine;

pub fn update(update_request_c: CUpdateRequest) callconv(.c) i32 {
    update_request_c.validate() catch |err| return @intFromEnum(fromError(err, Operation.update));

    const update_packages = collectUpdateEntries(update_request_c, ffi.getAllocator()) catch |err| return @intFromEnum(fromError(err, Operation.update));
    defer ffi.getAllocator().free(update_packages);

    const update_data = UpdateData{
        .packages = update_packages,
        .branch = update_request_c.branch.asZ(),
        .repo_path = update_request_c.repo_path.asZ(),
        .root_path = update_request_c.root_path.asZ(),
        .tmp_path = update_request_c.tmp_path.asZ(),
        .on_progress = if (update_request_c.on_progress) |cb| @as(UpdateProgressFn, @ptrCast(cb)) else null,
        .progress_ctx = update_request_c.progress_ctx,
        .cancel_token = update_request_c.cancel_token orelse return @intFromEnum(fromError(error.InvalidEntry, Operation.update)),
    };

    UpdateMachine.run(update_data, ffi.getAllocator()) catch |err| return @intFromEnum(fromError(err, Operation.update));

    return @intFromEnum(ErrorCode.ok);
}

fn collectUpdateEntries(update_request_c: CUpdateRequest, allocator: std.mem.Allocator) ![]const Package {
    if (update_request_c.packages_count > 0 and update_request_c.packages == null) return error.InvalidEntry;

    const packages_entry_c_ptr = update_request_c.packages orelse return error.InvalidEntry;

    const packages_entrys_c = packages_entry_c_ptr[0..update_request_c.packages_count];

    const update_packages = allocator.alloc(Package, packages_entrys_c.len) catch return error.OutOfMemory;
    errdefer allocator.free(update_packages);

    for (packages_entrys_c, 0..) |package_entry_c, index| {
        const package_meta_c: *CPackageMeta = @ptrCast(@alignCast(package_entry_c.meta));

        update_packages[index] = .{
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

    return update_packages;
}
