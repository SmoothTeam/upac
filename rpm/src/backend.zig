// ── Imports ─────────────────────────────────────────────────────────────────────
const states = @import("states.zig");
const stateFailed = states.stateFailed;

pub const std = @import("std");
pub const c_libs = @cImport({
    @cInclude("archive.h");
    @cInclude("archive_entry.h");
});

// ── Public types ────────────────────────────────────────────────────────────
pub const CancelToken = extern struct {
    _flag: u8,
    _hook: ?*const fn (ctx: ?*anyopaque) callconv(.c) void = null,
    _hook_ctx: ?*anyopaque = null,

    pub fn isCancelled(self: *const CancelToken) bool {
        return @atomicLoad(u8, &self._flag, .acquire) != 0;
    }
};
// Main structure containing package metadata
pub const PackageMeta = struct {
    name: []const u8,
    version: []const u8,
    size: u32,
    author: []const u8,
    description: []const u8,
    license: []const u8,
    arch: []const u8,
    url: []const u8,
    packager: []const u8,
    installed_at: i64,
    checksum: []const u8,
};

// Parameters for the package preparation request: paths to the archive and output folder, and the checksum
pub const PrepareRequest = struct {
    package_path: [*:0]const u8,
    temp_dir: [*:0]const u8,

    checksum: []const u8,

    on_progress: ?BackendProgressFn = null,
    progress_ctx: ?*anyopaque = null,

    cancel_token: *const CancelToken,
};

// Listing specific backend errors when working with archives and metadata
pub const BackendError = error{
    ChecksumMismatch,
    ExtractionFailed,
    MetadataNotFound,
    InvalidPackage,
    ReadFailed,
    ArchiveOpenFailed,
    ArchiveReadFailed,
    ArchiveExtractFailed,
    OutOfMemory,
    TempDirFailed,
    AllocZFailed,
    Cancelled,
};

// ── Internal FSM types ───────────────────────────────────────────────────────
// State Identifiers for the preparation process Finite State Machine (FSM)
pub const StateId = enum(u8) {
    verifying = 0,
    extracting = 1,
    reading_meta = 2,
    special_step = 3,

    done = 4,
    failed = 5,
};

// ── BackendFSM ───────────────────────────────────────────────────────
// A state machine context storing the transition stack, allocator, and parsing result
pub const BackendMachine = struct {
    request: PrepareRequest,
    cancel_token: *const CancelToken,
    meta: ?PackageMeta,

    temp_path: ?[:0]const u8 = null,
    file: ?std.Io.File = null,

    stack: std.ArrayList(StateId),
    io: std.Io,
    allocator: std.mem.Allocator,

    // Method for transitioning to a new state with history addition
    pub fn enter(self: *BackendMachine, state_id: StateId) !void {
        if (self.isCancelRequested()) {
            stateFailed(self);
            return BackendError.Cancelled;
        }
        try self.stack.append(self.allocator, state_id);
        self.report(state_id);
    }

    pub fn isCancelRequested(self: *const BackendMachine) bool {
        return self.cancel_token.isCancelled();
    }

    // Releasing resources (stack memory) occupied by the state machine
    pub fn deinit(self: *BackendMachine) void {
        if (self.temp_path) |path| self.allocator.free(path);
        if (self.file) |file| file.close(self.io);

        self.stack.deinit(self.allocator);
    }

    // Reports an installation progress event to the progress callback, if one is set
    pub fn report(self: *BackendMachine, event: StateId) void {
        const cb = self.request.on_progress orelse return;
        cb(event, CSlice.fromSlice(std.mem.span(self.request.package_path)), self.request.progress_ctx);
    }

    pub fn reportDetail(self: *BackendMachine, message: []const u8) void {
        const cb = self.request.on_progress orelse return;
        cb(.special_step, CSlice.fromSlice(message), self.request.progress_ctx);
    }

    pub inline fn unwrap(self: *BackendMachine, value: anytype, comptime err: BackendError) BackendError!@typeInfo(@TypeOf(value)).optional.child {
        return value orelse {
            stateFailed(self);
            return err;
        };
    }

    pub inline fn check(self: *BackendMachine, value: anytype, comptime err: BackendError) BackendError!@typeInfo(@TypeOf(value)).error_union.payload {
        return value catch {
            stateFailed(self);
            return err;
        };
    }

    // The entry and launch point of the machine, responsible for returning the correct result
    pub fn run(request: PrepareRequest, allocator: std.mem.Allocator) !PrepareResult {
        var machine = BackendMachine{
            .request = request,
            .cancel_token = request.cancel_token,
            .stack = std.ArrayList(StateId).empty,
            .io = std.Io.Threaded.global_single_threaded.io(),
            .allocator = allocator,
            .meta = null,
        };
        defer machine.deinit();
        try states.stateStart(&machine);

        const temp_path = try machine.unwrap(machine.temp_path, BackendError.TempDirFailed);
        machine.temp_path = null;

        return PrepareResult{
            .meta = try machine.unwrap(machine.meta, BackendError.InvalidPackage),
            .temp_path = temp_path,
        };
    }
};

// ── FFI ───────────────────────────────────────────────────────────────────────
// A helper structure for passing data slices via C FFI
const CSlice = extern struct {
    ptr: [*:0]const u8,
    len: usize,

    pub fn toSlice(self: CSlice) []const u8 {
        return self.ptr[0..self.len];
    }

    pub fn asZ(self: CSlice) [*:0]const u8 {
        return self.ptr;
    }

    pub fn fromSlice(slice: []const u8) CSlice {
        return .{ .ptr = @ptrCast(slice.ptr), .len = slice.len };
    }

    // A simple check to determine whether a passed string or data array is empty (i.e., has zero length)
    pub fn validate(self: CSlice) !void {
        if (self.ptr[self.len] != 0) return error.InvalidEntry;
        if (std.mem.len(self.ptr) != self.len) return error.InvalidEntry;
    }
};

// A C-compatible representation of package metadata for export to other languages
const CPackageMeta = extern struct {
    struct_size: usize = @sizeOf(CPackageMeta),

    name: CSlice,
    version: CSlice,
    arch: CSlice,
    author: CSlice,
    description: CSlice,
    license: CSlice,
    url: CSlice,
    packager: CSlice,
    checksum: CSlice,
    size: u32,
    _padding: u32 = 0,
    installed_at: i64,
};

// C-compatible request parameter structure for use in FFI
const CPrepareRequest = extern struct {
    struct_size: usize = @sizeOf(CPrepareRequest),
    checksum: CSlice,

    package_path: CSlice,
    temp_dir: CSlice,

    on_progress: ?CBackendProgressFn = null,
    progress_ctx: ?*anyopaque = null,

    cancel_token: ?*const CancelToken = null,

    pub fn validate(req: CPrepareRequest) !void {
        if (req.struct_size != @sizeOf(CPrepareRequest)) return error.AbiMismatch;

        try req.package_path.validate();
        try req.temp_dir.validate();
        try req.checksum.validate();
    }
};

pub const PrepareResult = struct {
    meta: PackageMeta,
    temp_path: [:0]const u8,
};

pub const BackendProgressFn = *const fn (
    event: StateId,
    package_name: CSlice,
    ctx: ?*anyopaque,
) callconv(.c) void;

pub const CBackendProgressFn = *const fn (
    event: StateId,
    package_name: CSlice,
    ctx: ?*anyopaque,
) callconv(.c) void;


// ── FFI экспорты ──────────────────────────────────────────────────────────────
// An exported C function (FFI) for initiating the preparation process from external code
pub export fn prepare(request_c: *const CPrepareRequest, out_meta: *?*anyopaque, out_temp_path: *CSlice) callconv(.c) i32 {
    request_c.validate() catch |err| return @intFromEnum(fromError(err));

    const cancel_token = request_c.cancel_token orelse return @intFromEnum(BackendErrorCode.invalid_entry);

    const zig_request = PrepareRequest{
        .package_path = request_c.package_path.asZ(),
        .temp_dir = request_c.temp_dir.asZ(),

        .checksum = request_c.checksum.toSlice(),

        .on_progress = request_c.on_progress,
        .progress_ctx = request_c.progress_ctx,

        .cancel_token = cancel_token,
    };

    const result = BackendMachine.run(zig_request, std.heap.c_allocator) catch |err| return @intFromEnum(fromError(err));

    const out_meta_ptr = std.heap.c_allocator.create(CPackageMeta) catch return @intFromEnum(BackendErrorCode.alloc_failed);

    out_meta_ptr.* = CPackageMeta{
        .struct_size = @sizeOf(CPackageMeta),

        .name = dupeRequiredToCSlice(std.heap.c_allocator, result.meta.name) catch return @intFromEnum(fromError(BackendError.InvalidPackage)),
        .version = dupeRequiredToCSlice(std.heap.c_allocator, result.meta.version) catch return @intFromEnum(fromError(BackendError.InvalidPackage)),
        .size = @intCast(result.meta.size),
        .arch = dupeToCSlice(std.heap.c_allocator, result.meta.arch) catch return @intFromEnum(fromError(BackendError.AllocZFailed)),
        .author = dupeToCSlice(std.heap.c_allocator, result.meta.author) catch return @intFromEnum(fromError(BackendError.AllocZFailed)),
        .description = dupeToCSlice(std.heap.c_allocator, result.meta.description) catch return @intFromEnum(fromError(BackendError.AllocZFailed)),
        .license = dupeToCSlice(std.heap.c_allocator, result.meta.license) catch return @intFromEnum(fromError(BackendError.AllocZFailed)),
        .url = dupeToCSlice(std.heap.c_allocator, result.meta.url) catch return @intFromEnum(fromError(BackendError.AllocZFailed)),
        .packager = dupeToCSlice(std.heap.c_allocator, result.meta.packager) catch return @intFromEnum(fromError(BackendError.AllocZFailed)),
        .installed_at = result.meta.installed_at,
        .checksum = dupeToCSlice(std.heap.c_allocator, result.meta.checksum) catch return @intFromEnum(fromError(BackendError.AllocZFailed)),
    };

    out_meta.* = out_meta_ptr;
    out_temp_path.* = dupeToCSlice(std.heap.c_allocator, result.temp_path) catch return @intFromEnum(fromError(BackendError.AllocZFailed));

    return @intFromEnum(BackendErrorCode.ok);
}

fn dupeToCSlice(allocator: std.mem.Allocator, slice: []const u8) BackendError!CSlice {
    const dupe_slice = allocator.dupe(u8, slice) catch return BackendError.AllocZFailed;
    return CSlice.fromSlice(dupe_slice);
}

fn dupeRequiredToCSlice(allocator: std.mem.Allocator, slice: []const u8) BackendError!CSlice {
    if (slice.len == 0) return BackendError.InvalidPackage;
    return dupeToCSlice(allocator, slice);
}

pub export fn cleanup(path_c: CSlice) callconv(.c) void {
    const path = path_c.toSlice();

    std.Io.Dir.cwd().deleteTree(std.Io.Threaded.global_single_threaded.io(), path) catch {};
    std.heap.c_allocator.free(path);
}

// A function for safely clearing metadata memory allocated on the Zig side
pub export fn meta_free(package_meta_c: *CPackageMeta) callconv(.c) void {
    std.heap.c_allocator.free(package_meta_c.name.toSlice());
    std.heap.c_allocator.free(package_meta_c.version.toSlice());
    std.heap.c_allocator.free(package_meta_c.arch.toSlice());
    std.heap.c_allocator.free(package_meta_c.author.toSlice());
    std.heap.c_allocator.free(package_meta_c.description.toSlice());
    std.heap.c_allocator.free(package_meta_c.license.toSlice());
    std.heap.c_allocator.free(package_meta_c.packager.toSlice());
    std.heap.c_allocator.free(package_meta_c.url.toSlice());
    std.heap.c_allocator.free(package_meta_c.checksum.toSlice());

    std.heap.c_allocator.destroy(package_meta_c);
}

pub export fn meta_get_name(meta: *const CPackageMeta) callconv(.c) CSlice {
    return meta.name;
}

pub export fn meta_get_version(meta: *const CPackageMeta) callconv(.c) CSlice {
    return meta.version;
}

pub const BackendErrorCode = enum(i32) {
    ok = 0,
    checksum_mismatch = 1,
    extraction_failed = 2,
    metadata_not_found = 3,
    invalid_package = 4,
    archive_open_failed = 5,
    archive_read_failed = 6,
    archive_extract_failed = 7,
    temp_dir_failed = 8,
    alloc_failed = 9,
    cancelled = 10,
    read_failed = 11,
    invalid_entry = 12,
    abi_mismatch = 13,
    unexpected = 99,
};

pub fn fromError(err: anyerror) BackendErrorCode {
    return switch (err) {
        BackendError.ChecksumMismatch => .checksum_mismatch,
        BackendError.ExtractionFailed => .extraction_failed,
        BackendError.MetadataNotFound => .metadata_not_found,
        BackendError.InvalidPackage => .invalid_package,
        BackendError.ArchiveOpenFailed => .archive_open_failed,
        BackendError.ArchiveReadFailed => .archive_read_failed,
        BackendError.ArchiveExtractFailed => .archive_extract_failed,
        BackendError.TempDirFailed => .temp_dir_failed,
        BackendError.AllocZFailed, BackendError.OutOfMemory => .alloc_failed,
        BackendError.Cancelled => .cancelled,
        BackendError.ReadFailed => .read_failed,
        error.InvalidEntry => .invalid_entry,
        error.AbiMismatch => .abi_mismatch,
        else => .unexpected,
    };
}

pub fn metaToC(allocator: std.mem.Allocator, meta: PackageMeta) BackendError!CPackageMeta {
    var result = CPackageMeta{
        .size = @intCast(meta.size),
        .installed_at = meta.installed_at,
        .name = .{ .ptr = null, .len = 0 },
        .version = .{ .ptr = null, .len = 0 },
        .arch = .{ .ptr = null, .len = 0 },
        .author = .{ .ptr = null, .len = 0 },
        .description = .{ .ptr = null, .len = 0 },
        .license = .{ .ptr = null, .len = 0 },
        .url = .{ .ptr = null, .len = 0 },
        .packager = .{ .ptr = null, .len = 0 },
        .checksum = .{ .ptr = null, .len = 0 },
    };

    const string_fields = .{
        .{ "name", meta.name },
        .{ "version", meta.version },
        .{ "arch", meta.arch },
        .{ "author", meta.author },
        .{ "description", meta.description },
        .{ "license", meta.license },
        .{ "url", meta.url },
        .{ "packager", meta.packager },
        .{ "checksum", meta.checksum },
    };

    inline for (string_fields) |pair| {
        @field(result, pair[0]) = try dupeToCSlice(allocator, pair[1]);
    }

    return result;
}
