// ── Imports ──────────────────────────────────────────────────────────
const std = @import("std");

const c_libs = @import("c-libs");

const types = @import("upac-types");
const PackageMeta = types.PackageMeta;
const Version = types.Version;
const DiffKind = types.DiffKind;
const DiffStateId = types.DiffStateId;
const DiffError = types.DiffError;

const ffi = @import("upac-ffi");
const CDiffPackageEntry = ffi.CDiffPackageEntry;
const CSlice = ffi.CSlice;
const CArray = ffi.CArray;
const CancelToken = ffi.CancelToken;
const cancelGCancellable = ffi.cancelGCancellable;

const verifying = @import("verifying/verifying.zig");
const preparation = @import("preparation/preparation.zig");
const comparing = @import("comparing/comparing.zig");

pub const PackageDiffEntry = struct {
    name: []const u8,
    kind: DiffKind,
    version: Version,
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

    packages_lists: [2]std.ArrayList(PackageMeta) = undefined,

    cancellable: ?*c_libs.GCancellable = null,
    gerror: ?*c_libs.GError = null,

    allocator: std.mem.Allocator,
    io: std.Io,

    pub fn check(self: *DiffMachine, result: c_int, fallback: DiffError) DiffError!void {
        if (result != 0) return;
        defer {
            if (self.gerror) |err| {
                c_libs.g_error_free(err);
                self.gerror = null;
            }
        }
        if (self.gerror) |err| {
            if (err.domain == c_libs.g_io_error_quark() and err.code == c_libs.G_IO_ERROR_CANCELLED)
                return DiffError.Cancelled;
        }
        return fallback;
    }

    pub fn deinit(self: *DiffMachine) void {
        for (&self.packages_lists) |*list| {
            for (list.items) |*meta| meta.deinit(self.allocator);
            list.deinit(self.allocator);
        }
        if (self.repo) |repo| c_libs.g_object_unref(repo);
        if (self.gerror) |err| c_libs.g_error_free(err);
        if (self.cancellable) |cancellable| c_libs.g_object_unref(cancellable);
    }

    pub fn run(diff_data: DiffData, allocator: std.mem.Allocator) DiffError![]CDiffPackageEntry {
        var state = DiffStateId.verifying;
        var entries: []PackageDiffEntry = &.{};
        errdefer if (entries.len > 0) {
            for (entries) |entry| {
                allocator.free(entry.name);
                entry.version.deinit(allocator);
            }
            allocator.free(entries);
        };

        var machine = DiffMachine{
            .data = diff_data,

            .packages_lists = .{
                std.ArrayList(PackageMeta).empty,
                std.ArrayList(PackageMeta).empty,
            },

            .cancellable = c_libs.g_cancellable_new() orelse return DiffError.Cancelled,

            .allocator = allocator,
            .io = std.Io.Threaded.global_single_threaded.io(),
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
                    entries = comparing.run(&machine) catch |err| return err;
                    state = .done;
                },
                .done => {},
            }
        }

        const c_entries = allocator.alloc(CDiffPackageEntry, entries.len) catch return DiffError.AllocFailed;

        for (entries, c_entries) |entry, *c_entry| {
            c_entry.* = .{
                .name = CSlice.fromSlice(entry.name),
                .kind = entry.kind,
                .version = .{
                    .epoch = entry.version.epoch,
                    .release = entry.version.release,
                    .parts = .{ .ptr = @constCast(entry.version.parts.ptr), .len = entry.version.parts.len },
                    .pre = CSlice.fromSlice(entry.version.pre),
                },
            };
        }
        allocator.free(entries);

        return c_entries;
    }
};
