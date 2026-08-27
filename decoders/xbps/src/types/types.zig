// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

pub const std = @import("std");

const meta_fields = @import("upac-meta-fields");

const errors = @import("errors.zig");
pub const BackendErrorCode = errors.BackendErrorCode;
pub const BackendError = errors.BackendError;
pub const fromError = errors.fromError;

// ── StateId ───────────────────────────────────────────────────────────────────
pub const StateId = enum(u8) {
    verifying = 0,
    reading_meta = 1,
    extracting = 2,
    special_step = 3,

    done = 4,
};

// ── Version ─────────────────────────────────────────────────────────────────
pub const Version = struct {
    epoch: u32 = 0,
    parts: []const u32,
    pre: ?[]const u8 = null,
    release: u32 = 1,

    pub fn deinit(self: *const Version, allocator: std.mem.Allocator) void {
        allocator.free(self.parts);
        if (self.pre) |pre| allocator.free(pre);
    }
};

// ── Hook ──────────────────────────────────────────────────────────────────────
pub const HookResponse = enum(u8) {
    proceed = 0,
    cancel = 1,
};

pub const HookFn = fn (event: u32, data: ?*const anyopaque, ctx: ?*anyopaque) callconv(.c) HookResponse;

// ── CancelToken ───────────────────────────────────────────────────────────────
pub const CancelToken = extern struct {
    _flag: u8,
    _hook: ?*const fn (ctx: ?*anyopaque) callconv(.c) void = null,
    _hook_ctx: ?*anyopaque = null,

    pub fn isCancelled(self: *const CancelToken) bool {
        return @atomicLoad(u8, &self._flag, .acquire) != 0;
    }
};

// ── plist_field_map ───────────────────────────────────────────────────────────
const RawMetaField = std.meta.FieldEnum(RawMeta);

pub const plist_field_map = blk: {
    const zon_fields = std.meta.fields(@TypeOf(meta_fields));
    var entries: [zon_fields.len]struct { []const u8, RawMetaField } = undefined;
    for (zon_fields, 0..) |field, index| {
        const raw_meta_field_name = @field(meta_fields, field.name);
        entries[index] = .{ field.name, @field(RawMetaField, raw_meta_field_name) };
    }
    break :blk std.StaticStringMap(RawMetaField).initComptime(entries);
};

// ── PrepareData ───────────────────────────────────────────────────────────────
pub const PrepareData = struct {
    package_path_c: [*:0]const u8,
    temp_path_c: [*:0]const u8,
    checksum: []const u8,
    on_hook: ?*const HookFn = null,
    hook_ctx: ?*anyopaque = null,
    cancel_token: *const CancelToken,
};

// ── RawMeta ───────────────────────────────────────────────────────────────────
pub const RawMeta = struct {
    name: ?[]const u8 = null,
    version: ?[]const u8 = null,
    arch: ?[]const u8 = null,
    size: u32 = 0,
    description: ?[]const u8 = null,
    url: ?[]const u8 = null,
    packager: ?[]const u8 = null,
    license: ?[]const u8 = null,

    pub fn deinit(self: *RawMeta, allocator: std.mem.Allocator) void {
        if (self.name) |value| allocator.free(value);
        if (self.version) |value| allocator.free(value);
        if (self.arch) |value| allocator.free(value);
        if (self.description) |value| allocator.free(value);
        if (self.url) |value| allocator.free(value);
        if (self.packager) |value| allocator.free(value);
        if (self.license) |value| allocator.free(value);
    }
};

// ── PackageMeta ───────────────────────────────────────────────────────────────
pub const PackageMeta = struct {
    name: []const u8,
    version: Version,
    arch: []const u8,
    author: []const u8,
    description: []const u8,
    license: []const u8,
    url: []const u8,
    packager: []const u8,
    checksum: [32]u8,
    size: u32,
    installed_at: i64,

    pub fn deinit(self: *PackageMeta, allocator: std.mem.Allocator) void {
        allocator.free(self.name);
        allocator.free(self.arch);
        allocator.free(self.author);
        allocator.free(self.description);
        allocator.free(self.license);
        allocator.free(self.url);
        allocator.free(self.packager);
        self.version.deinit(allocator);
    }
};

// ── PrepareResult ─────────────────────────────────────────────────────────────
pub const PrepareResult = struct {
    meta: PackageMeta,
    temp_path: [:0]const u8,
};
