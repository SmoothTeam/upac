// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later

// ── Imports ───────────────────────────────────────────────────────────────────
const std = @import("std");

const types = @import("upac-backend-types");

const BackendError = types.BackendError;

const PackageMeta = types.PackageMeta;
const RawMeta = types.RawMeta;

const plist_field_map = types.plist_field_map;

const backend = @import("../backend.zig");
const Machine = backend.BackendMachine;

const utils = @import("utils.zig");
const decodeXmlEntities = utils.decodeXmlEntities;
const parseVersion = utils.parseVersion;

// ── ParsingState ──────────────────────────────────────────────────────────────
const ParsingState = enum {
    parse_plist,
    build_meta,
    done,
};

// ── ParsingMachine ────────────────────────────────────────────────────────────
const RawMetaField = std.meta.FieldEnum(RawMeta);

const ParsingMachine = struct {
    backend: *Machine,

    raw_meta: RawMeta = .{},

    fn stateFailed(self: *ParsingMachine, err: BackendError) BackendError {
        self.raw_meta.deinit(self.backend.allocator);

        if (self.backend.props_content) |content| {
            self.backend.allocator.free(content);
            self.backend.props_content = null;
        }

        return err;
    }
};

// ── Trampoline ────────────────────────────────────────────────────────────────
pub fn run(machine: *Machine) BackendError!void {
    var parsing = ParsingMachine{ .backend = machine };

    var state = ParsingState.parse_plist;
    while (state != .done) {
        if (machine.data.cancel_token.isCancelled()) return parsing.stateFailed(BackendError.Cancelled);
        state = switch (state) {
            .parse_plist => try stateParsePlist(&parsing),
            .build_meta => try stateBuildMeta(&parsing),
            .done => unreachable,
        };
    }
}

// ── States ────────────────────────────────────────────────────────────────────
fn stateParsePlist(machine: *ParsingMachine) BackendError!ParsingState {
    const props_content = machine.backend.props_content orelse return machine.stateFailed(BackendError.MetadataNotFound);
    machine.backend.props_content = null;
    defer machine.backend.allocator.free(props_content);

    var pos: usize = 0;
    while (pos < props_content.len) {
        const key_open = std.mem.indexOfPos(u8, props_content, pos, "<key>") orelse break;

        const key_start = key_open + "<key>".len;
        const key_close = std.mem.indexOfPos(u8, props_content, key_start, "</key>") orelse break;
        const key = props_content[key_start..key_close];
        pos = key_close + "</key>".len;

        while (pos < props_content.len and (props_content[pos] == ' ' or props_content[pos] == '\t' or
            props_content[pos] == '\n' or props_content[pos] == '\r')) pos += 1;

        const field_kind = plist_field_map.get(key) orelse {
            pos += 1;
            continue;
        };

        if (std.mem.startsWith(u8, props_content[pos..], "<string>")) {
            const val_start = pos + "<string>".len;
            const val_close = std.mem.indexOfPos(u8, props_content, val_start, "</string>") orelse break;
            const value = props_content[val_start..val_close];
            pos = val_close + "</string>".len;

            const decoded = decodeXmlEntities(machine.backend.allocator, value) catch return machine.stateFailed(BackendError.OutOfMemory);
            switch (field_kind) {
                .name => machine.raw_meta.name = decoded,
                .version => machine.raw_meta.version = decoded,
                .arch => machine.raw_meta.arch = decoded,
                .description => machine.raw_meta.description = decoded,
                .url => machine.raw_meta.url = decoded,
                .packager => machine.raw_meta.packager = decoded,
                .license => machine.raw_meta.license = decoded,
                .size => machine.backend.allocator.free(decoded),
            }
        } else if (std.mem.startsWith(u8, props_content[pos..], "<integer>")) {
            const val_start = pos + "<integer>".len;
            const val_close = std.mem.indexOfPos(u8, props_content, val_start, "</integer>") orelse break;
            const value = props_content[val_start..val_close];
            pos = val_close + "</integer>".len;

            switch (field_kind) {
                .size => machine.raw_meta.size = std.fmt.parseInt(u32, value, 10) catch 0,
                else => {},
            }
        } else pos += 1;
    }

    return .build_meta;
}

fn stateBuildMeta(machine: *ParsingMachine) BackendError!ParsingState {
    var sha256: [32]u8 = undefined;

    _ = std.fmt.hexToBytes(&sha256, machine.backend.data.checksum) catch return machine.stateFailed(BackendError.InvalidPackage);

    const raw_version_str = machine.raw_meta.version orelse return machine.stateFailed(BackendError.MetadataNotFound);
    defer machine.backend.allocator.free(raw_version_str);
    machine.raw_meta.version = null;

    const parsed_version = parseVersion(machine.backend.allocator, raw_version_str) catch return machine.stateFailed(BackendError.InvalidPackage);
    errdefer parsed_version.deinit(machine.backend.allocator);

    const package_name = machine.raw_meta.name orelse return machine.stateFailed(BackendError.MetadataNotFound);
    errdefer machine.backend.allocator.free(package_name);
    machine.raw_meta.name = null;

    const package_arch = machine.raw_meta.arch orelse machine.backend.allocator.dupe(u8, "") catch return machine.stateFailed(BackendError.OutOfMemory);
    errdefer machine.backend.allocator.free(package_arch);
    machine.raw_meta.arch = null;

    const package_description = machine.raw_meta.description orelse machine.backend.allocator.dupe(u8, "") catch return machine.stateFailed(BackendError.OutOfMemory);
    errdefer machine.backend.allocator.free(package_description);
    machine.raw_meta.description = null;

    const package_url = machine.raw_meta.url orelse machine.backend.allocator.dupe(u8, "") catch return machine.stateFailed(BackendError.OutOfMemory);
    errdefer machine.backend.allocator.free(package_url);
    machine.raw_meta.url = null;

    const package_packager = machine.raw_meta.packager orelse machine.backend.allocator.dupe(u8, "") catch return machine.stateFailed(BackendError.OutOfMemory);
    errdefer machine.backend.allocator.free(package_packager);
    machine.raw_meta.packager = null;

    const package_author = machine.backend.allocator.dupe(u8, package_packager) catch return machine.stateFailed(BackendError.OutOfMemory);
    errdefer machine.backend.allocator.free(package_author);

    const package_license = machine.raw_meta.license orelse machine.backend.allocator.dupe(u8, "") catch return machine.stateFailed(BackendError.OutOfMemory);
    errdefer machine.backend.allocator.free(package_license);
    machine.raw_meta.license = null;

    machine.backend.meta = PackageMeta{
        .name = package_name,
        .version = parsed_version,
        .arch = package_arch,
        .author = package_author,
        .description = package_description,
        .license = package_license,
        .url = package_url,
        .packager = package_packager,
        .checksum = sha256,
        .size = machine.raw_meta.size,
        .installed_at = @intCast(@divTrunc(std.Io.Clock.real.now(machine.backend.io).nanoseconds, std.time.ns_per_s)),
    };

    machine.raw_meta.deinit(machine.backend.allocator);

    return .done;
}
