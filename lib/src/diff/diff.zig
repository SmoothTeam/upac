// ── Imports ──────────────────────────────────────────────────────────
const std = @import("std");

const c_libs = @import("c-libs");

const ffi = @import("upac-ffi");

const types = @import("upac-types");
const FileRecord = types.FileRecord;

const DiffStateId = types.DiffStateId;

const CDiffEntry = ffi.CDiffEntry;
const CSlice = ffi.CSlice;

const CancelToken = ffi.CancelToken;
const cancelGCancellable = ffi.cancelGCancellable;

const verifying = @import("verifying/verifying.zig");
const preparation = @import("preparation/preparation.zig");
const comparing = @import("comparing/comparing.zig");
// ── Errors ───────────────────────────────────────────────────────────────────
pub const DiffError = error{
    RepoOpenFailed,
    CommitNotFound,
    DiffFailed,
    AllocFailed,
    OutOfMemory,
    FileNotFound,
    PathNotFound,
    CheckSpaceFailed,
    NotEnoughSpace,
    CheckoutFailed,
    ReadDatabaseFailed,
    Cancelled,
};

pub const DiffData = struct {
    repo_path: [*:0]const u8,
    tmp_path: [*:0]const u8,

    from_ref: [*:0]const u8,
    to_ref: [*:0]const u8,

    cancel_token: *CancelToken,
};

pub const DiffMachine = struct {
    data: DiffData,
    repo: ?*c_libs.OstreeRepo = null,

    file_pkg_maps: [2]std.StringHashMap(FileRecord) = undefined,

    cancellable: ?*c_libs.GCancellable = null,
    gerror: ?*c_libs.GError = null,

    allocator: std.mem.Allocator,
    io: std.Io,

    pub fn check(self: *DiffMachine) DiffError!void {
        if (self.gerror) |err| {
            const is_cancel = err.domain == c_libs.g_io_error_quark() and err.code == c_libs.G_IO_ERROR_CANCELLED;

            c_libs.g_error_free(err);
            self.gerror = null;

            return if (is_cancel) DiffError.Cancelled else DiffError.DiffFailed;
        }

        if (self.cancellable) |cancellable| {
            if (c_libs.g_cancellable_is_cancelled(cancellable) != 0) {
                c_libs.g_cancellable_cancel(cancellable);
                return DiffError.Cancelled;
            }
        }
    }

    pub fn deinit(self: *DiffMachine) void {
        for (&self.file_pkg_maps) |*map| {
            var iter = map.iterator();
            while (iter.next()) |entry| {
                self.allocator.free(entry.key_ptr.*);
                self.allocator.free(entry.value_ptr.*.pkg_name);
            }
            map.deinit();
        }
        if (self.repo) |repo| c_libs.g_object_unref(repo);
        if (self.gerror) |err| c_libs.g_error_free(err);
        if (self.cancellable) |cancellable| c_libs.g_object_unref(cancellable);
    }

    pub fn run(diff_data: DiffData, allocator: std.mem.Allocator) DiffError![]CDiffEntry {
        var state = DiffStateId.verifying;

        var machine = DiffMachine{
            .data = diff_data,

            .cancellable = c_libs.g_cancellable_new() orelse return DiffError.Cancelled,

            .allocator = allocator,
            .io = std.Io.Threaded.global_single_threaded.io(),

            .file_pkg_maps = .{
                std.StringHashMap(FileRecord).init(allocator),
                std.StringHashMap(FileRecord).init(allocator),
            },
        };
        defer machine.deinit();

        diff_data.cancel_token.hook = cancelGCancellable;
        diff_data.cancel_token.hook_ctx = machine.cancellable;
        defer diff_data.cancel_token.reset();

        while (state != .done) {
            switch (state) {
                .verifying => {
                    verifying.run(&machine) catch |err| return err;
                    state = .preparing;
                },
                .preparing => {
                    preparation.run(&machine) catch |err| return err;
                    state = .comparing;
                },
                .comparing => {
                    const entries = comparing.run(&machine) catch |err| return err;

                    const c_entries = allocator.alloc(CDiffEntry, entries.len) catch {
                        for (entries) |entry| {
                            allocator.free(entry.path);
                            allocator.free(entry.package_name);
                        }
                        allocator.free(entries);
                        return DiffError.AllocFailed;
                    };

                    for (entries, c_entries) |entry, *c_entry| {
                        c_entry.* = .{
                            .path = CSlice.fromSlice(entry.path),
                            .kind = entry.kind,
                            .package_name = CSlice.fromSlice(entry.package_name),
                            .is_user = entry.is_user,
                        };
                    }
                    allocator.free(entries);

                    return c_entries;
                },
                .done => state = .done,
                .failed => state = .failed,
            }
        }

        return &{};
    }
};
