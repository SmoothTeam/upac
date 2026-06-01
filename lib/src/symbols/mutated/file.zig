const ffi = @import("upac-ffi");
const CSlice = ffi.CSlice;

const CFilesRequest = ffi.CMutatedRequest;
const CPackageInfo = ffi.CPackageInfo;
const HookFn = ffi.HookFn;

const types = @import("upac-types");
const DiffKind = types.DiffKind;
const ErrorCode = types.ErrorCode;
const Operation = types.Operation;
const fromError = types.fromError;

const files_module = @import("upac-files");
const FilesData = files_module.FilesData;
const FilesMachine = files_module.FilesMachine;

pub fn files(request_c: CFilesRequest) callconv(.c) i32 {
    request_c.validate() catch |err| return @intFromEnum(fromError(err, Operation.files));

    const files_c_ptr = request_c.files orelse return @intFromEnum(fromError(error.InvalidEntry, Operation.files));
    if (request_c.files_len == 0) return @intFromEnum(fromError(error.InvalidEntry, Operation.files));
    const files_c = files_c_ptr[0..request_c.files_len];
    for (files_c) |f| if (f.ptr == null or f.len == 0 or f.ptr[f.len] != 0) return @intFromEnum(fromError(error.InvalidEntry, Operation.files));

    _ = ffi.intToEnum(DiffKind, @intFromEnum(request_c.file_kind)) catch return @intFromEnum(fromError(error.InvalidEntry, Operation.files));

    const package_c = request_c.file_package orelse return @intFromEnum(fromError(error.InvalidEntry, Operation.files));
    package_c.validate() catch return @intFromEnum(fromError(error.InvalidEntry, Operation.files));

    const file_paths = ffi.getAllocator().alloc([*:0]const u8, files_c.len) catch return @intFromEnum(ErrorCode.out_of_memory);
    defer ffi.getAllocator().free(file_paths);
    for (files_c, file_paths) |f, *p| p.* = f.asZ();

    const files_data = FilesData{
        .file_paths = file_paths,
        .kind = request_c.file_kind,

        .pkg_name = package_c.name.asZ(),
        .pkg_arch = package_c.arch.asZ(),
        .pkg_arch_sub = if (package_c.arch_sub.ptr != null) package_c.arch_sub.asZ() else null,

        .repo_path = request_c.repo_path.asZ(),
        .root_path = request_c.root_path.asZ(),
        .tmp_path = request_c.tmp_path.asZ(),
        .branch = request_c.branch.asZ(),

        .on_hook = request_c.on_hook,
        .hook_ctx = request_c.hook_ctx,

        .cancel_token = request_c.cancel_token orelse return @intFromEnum(fromError(error.InvalidEntry, Operation.files)),
    };

    FilesMachine.run(files_data, ffi.getAllocator()) catch |err| return @intFromEnum(fromError(err, Operation.files));

    return @intFromEnum(ErrorCode.ok);
}
