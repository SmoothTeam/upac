const std = @import("std");

const ffi = @import("upac-ffi");
const CPackageMeta = ffi.CPackageMeta;

const types = @import("upac-types");
const ListPackagesStateId = types.ListPackagesStateId;

const ListError = types.ListError;

const verifying = @import("verifying/verifying.zig");
const fetch = @import("fetch/fetch.zig");

// ── ListData ──────────────────────────────────────────────────────────────────
pub const ListData = struct {
    root_path: [*:0]const u8,
};

// ── ListMachine ───────────────────────────────────────────────────────────────
pub const ListMachine = struct {
    data: ListData,

    allocator: std.mem.Allocator,
    io: std.Io,

    pub fn run(list_data: ListData, allocator: std.mem.Allocator) ListError![]CPackageMeta {
        var packages: []CPackageMeta = &.{};
        var state = ListPackagesStateId.verifying;

        var machine = ListMachine{
            .data = list_data,
            .allocator = allocator,
            .io = std.Io.Threaded.global_single_threaded.io(),
        };

        while (state != .done) {
            switch (state) {
                .verifying => {
                    verifying.run(&machine) catch |err| return err;
                    state = .fetching;
                },
                .fetching => {
                    packages = fetch.run(&machine) catch |err| return err;
                    state = .done;
                },
                .done => {},
            }
        }

        return packages;
    }
};
