const std = @import("std");

const ffi = @import("upac-ffi");
const CancelToken = ffi.CancelToken;
const CPackageMeta = ffi.CPackageMeta;

const types = @import("upac-types");
const SearchMetaStateId = types.SearchMetaStateId;

const verifying = @import("verifying/verifying.zig");
const searching = @import("searching/searching.zig");

// ── Errors ────────────────────────────────────────────────────────────────────
pub const SearchMetaError = error{
    PathNotFound,
    ReadDatabaseFailed,
    AllocZFailed,
    OutOfMemory,
    Cancelled,
};

// ── SearchMetaData ────────────────────────────────────────────────────────────
pub const SearchMetaData = struct {
    root_path: [*:0]const u8,

    query: []const u8,

    cancel_token: *CancelToken,
};

// ── SearchMetaMachine ─────────────────────────────────────────────────────────
pub const SearchMetaMachine = struct {
    data: SearchMetaData,

    results: ?[]CPackageMeta = null,

    allocator: std.mem.Allocator,
    io: std.Io,

    pub fn run(data: SearchMetaData, allocator: std.mem.Allocator) SearchMetaError![]CPackageMeta {
        var machine = SearchMetaMachine{
            .data = data,

            .allocator = allocator,
            .io = std.Io.Threaded.global_single_threaded.io(),
        };

        var state = SearchMetaStateId.verifying;
        while (state != .done) {
            switch (state) {
                .verifying => {
                    verifying.run(&machine) catch |err| return err;
                    state = .searching;
                },
                .searching => {
                    searching.run(&machine) catch |err| return err;
                    state = .done;
                },
                .done => {},
            }
        }

        return machine.results orelse &.{};
    }
};
