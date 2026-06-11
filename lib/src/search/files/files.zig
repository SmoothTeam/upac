const std = @import("std");

const ffi = @import("upac-ffi");
const CancelToken = ffi.CancelToken;
const CDiffFileEntry = ffi.CDiffFileEntry;

const types = @import("upac-types");
const SearchFilesStateId = types.SearchFilesStateId;

const verifying = @import("verifying/verifying.zig");
const searching = @import("search/search.zig");

// ── Errors ────────────────────────────────────────────────────────────────────
pub const SearchFilesError = error{
    PathNotFound,
    ReadDatabaseFailed,
    AllocZFailed,
    OutOfMemory,
    Cancelled,
    AccessDenied,
};

// ── SearchFilesData ───────────────────────────────────────────────────────────
pub const SearchFilesData = struct {
    root_path: [*:0]const u8,
    query: []const u8,
    cancel_token: *CancelToken,
};

// ── SearchFilesMachine ────────────────────────────────────────────────────────
pub const SearchFilesMachine = struct {
    data: SearchFilesData,

    results: ?[]CDiffFileEntry = null,

    allocator: std.mem.Allocator,
    io: std.Io,

    pub fn run(data: SearchFilesData, allocator: std.mem.Allocator) SearchFilesError![]CDiffFileEntry {
        var machine = SearchFilesMachine{
            .data = data,

            .allocator = allocator,
            .io = std.Io.Threaded.global_single_threaded.io(),
        };

        var state = SearchFilesStateId.verifying;
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
