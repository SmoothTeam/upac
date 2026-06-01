const std = @import("std");

const c_libs = @import("c-libs");

const ffi = @import("upac-ffi");
const CCommitEntry = ffi.CCommitEntry;
const CancelToken = ffi.CancelToken;
const cancelGCancellable = ffi.cancelGCancellable;

const types = @import("upac-types");
const CommitStateId = types.CommitStateId;

const ListError = types.ListError;

const verifying = @import("verifying/verifying.zig");
const fetch = @import("fetch/fetch.zig");

// ── CommitEntry ───────────────────────────────────────────────────────────────
pub const CommitEntry = struct {
    checksum: []u8,
    subject: []u8,

    pub fn deinit(self: CommitEntry, allocator: std.mem.Allocator) void {
        allocator.free(self.checksum);
        allocator.free(self.subject);
    }
};

// ── CommitData ────────────────────────────────────────────────────────────────
pub const CommitData = struct {
    repo_path: [*:0]const u8,
    root_path: [*:0]const u8,
    branch: [*:0]const u8,

    cancel_token: *CancelToken,
};

// ── CommitMachine ─────────────────────────────────────────────────────────────
pub const CommitMachine = struct {
    data: CommitData,

    cancellable: ?*c_libs.GCancellable = null,
    gerror: ?*c_libs.GError = null,

    allocator: std.mem.Allocator,
    io: std.Io,

    pub fn deinit(self: *CommitMachine) void {
        if (self.gerror) |err| c_libs.g_error_free(err);
        if (self.cancellable) |cancellable| c_libs.g_object_unref(cancellable);
    }

    pub fn run(commit_data: CommitData, allocator: std.mem.Allocator) ListError![]CCommitEntry {
        var commits: []CCommitEntry = &.{};
        var state = CommitStateId.verifying;

        var machine = CommitMachine{
            .data = commit_data,
            .cancellable = c_libs.g_cancellable_new() orelse return ListError.OutOfMemory,
            .allocator = allocator,
            .io = std.Io.Threaded.global_single_threaded.io(),
        };
        defer machine.deinit();

        commit_data.cancel_token.hook = cancelGCancellable;
        commit_data.cancel_token.hook_ctx = machine.cancellable;
        defer commit_data.cancel_token.reset();

        while (state != .done) {
            switch (state) {
                .verifying => {
                    verifying.run(&machine) catch |err| return err;
                    state = .fetching;
                },
                .fetching => {
                    commits = fetch.run(&machine) catch |err| return err;
                    state = .done;
                },
                .done => {},
            }
        }

        return commits;
    }
};
