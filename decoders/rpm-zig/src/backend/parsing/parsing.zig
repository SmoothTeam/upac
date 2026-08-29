// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

// ── Imports ───────────────────────────────────────────────────────────────────
const std = backend.std;

const types = @import("upac-backend-types");
const rpm_lead_size = types.rpm_lead_size;

const BackendError = types.BackendError;

const PackageMeta = types.PackageMeta;
const RawMeta = types.RawMeta;

const RpmTag = types.RpmTag;

const backend = @import("../backend.zig");
const Machine = backend.BackendMachine;

const utils = @import("utils.zig");
const readExact = utils.readExact;
const readString = utils.readString;
const parseVersion = utils.parseVersion;

// ── Constants ─────────────────────────────────────────────────────────────────
const header_magic = [3]u8{ 0x8E, 0xAD, 0xE8 };

// ── ParsingState ──────────────────────────────────────────────────────────────
const ParsingState = enum {
    skip_lead,
    skip_signature,
    read_header_index,
    read_data_store,
    extract_tags,
    build_meta,
    done,
};

// ── ParsingMachine ────────────────────────────────────────────────────────────
const ParsingMachine = struct {
    backend: *Machine,

    file: ?std.Io.File = null,

    index_bytes: ?[]u8 = null,
    data_block: ?[]u8 = null,

    tag_count: u32 = 0,
    data_size: u32 = 0,

    raw_meta: RawMeta = .{},

    fn stateFailed(self: *ParsingMachine, err: BackendError) BackendError {
        if (self.file) |file| {
            file.close(self.backend.io);
            self.file = null;
        }

        if (self.index_bytes) |bytes| {
            self.backend.allocator.free(bytes);
            self.index_bytes = null;
        }

        if (self.data_block) |block| {
            self.backend.allocator.free(block);
            self.data_block = null;
        }

        self.raw_meta.deinit(self.backend.allocator);
        return err;
    }
};

// ── Trampoline ────────────────────────────────────────────────────────────────
pub fn run(machine: *Machine) BackendError!void {
    var parsing = ParsingMachine{ .backend = machine };

    var state = ParsingState.skip_lead;
    while (state != .done) {
        if (machine.data.cancel_token.isCancelled()) return parsing.stateFailed(BackendError.Cancelled);
        state = switch (state) {
            .skip_lead => try stateSkipLead(&parsing),
            .skip_signature => try stateSkipSignature(&parsing),
            .read_header_index => try stateReadHeaderIndex(&parsing),
            .read_data_store => try stateReadDataStore(&parsing),
            .extract_tags => try stateExtractTags(&parsing),
            .build_meta => try stateBuildMeta(&parsing),
            .done => unreachable,
        };
    }
}

// ── States ────────────────────────────────────────────────────────────────────
fn stateSkipLead(machine: *ParsingMachine) BackendError!ParsingState {
    const package_path = std.mem.span(machine.backend.data.package_path_c);

    const file = std.Io.Dir.openFileAbsolute(machine.backend.io, package_path, .{}) catch return machine.stateFailed(BackendError.ReadFailed);
    machine.file = file;

    machine.backend.io.vtable.fileSeekTo(machine.backend.io.userdata, file, rpm_lead_size) catch return machine.stateFailed(BackendError.ReadFailed);

    return .skip_signature;
}

fn stateSkipSignature(machine: *ParsingMachine) BackendError!ParsingState {
    var section_header_buf: [16]u8 = undefined;

    const file = machine.file orelse return machine.stateFailed(BackendError.ReadFailed);

    readExact(machine.backend.io, file, &section_header_buf) catch return machine.stateFailed(BackendError.InvalidPackage);

    if (!std.mem.eql(u8, section_header_buf[0..3], &header_magic)) return machine.stateFailed(BackendError.InvalidPackage);

    const signature_tag_count = std.mem.readInt(u32, section_header_buf[8..12], .big);
    const signature_data_size = std.mem.readInt(u32, section_header_buf[12..16], .big);

    const signature_total_size: u64 = @as(u64, signature_tag_count) * 16 + signature_data_size;
    machine.backend.io.vtable.fileSeekBy(machine.backend.io.userdata, file, @intCast(signature_total_size)) catch return machine.stateFailed(BackendError.ReadFailed);

    const alignment_remainder = signature_total_size % 8;
    if (alignment_remainder != 0) machine.backend.io.vtable.fileSeekBy(machine.backend.io.userdata, file, @intCast(8 - alignment_remainder)) catch return machine.stateFailed(BackendError.ReadFailed);

    return .read_header_index;
}

fn stateReadHeaderIndex(machine: *ParsingMachine) BackendError!ParsingState {
    var section_header_buf: [16]u8 = undefined;

    const file = machine.file orelse return machine.stateFailed(BackendError.ReadFailed);

    readExact(machine.backend.io, file, &section_header_buf) catch return machine.stateFailed(BackendError.InvalidPackage);

    if (!std.mem.eql(u8, section_header_buf[0..3], &header_magic)) return machine.stateFailed(BackendError.InvalidPackage);

    machine.tag_count = std.mem.readInt(u32, section_header_buf[8..12], .big);
    machine.data_size = std.mem.readInt(u32, section_header_buf[12..16], .big);

    const index_bytes = machine.backend.allocator.alloc(u8, @as(usize, machine.tag_count) * 16) catch return machine.stateFailed(BackendError.OutOfMemory);
    machine.index_bytes = index_bytes;

    readExact(machine.backend.io, file, index_bytes) catch return machine.stateFailed(BackendError.InvalidPackage);

    return .read_data_store;
}

fn stateReadDataStore(machine: *ParsingMachine) BackendError!ParsingState {
    const file = machine.file orelse return machine.stateFailed(BackendError.ReadFailed);

    const data_block = machine.backend.allocator.alloc(u8, machine.data_size) catch return machine.stateFailed(BackendError.OutOfMemory);
    machine.data_block = data_block;

    readExact(machine.backend.io, file, data_block) catch return machine.stateFailed(BackendError.InvalidPackage);

    file.close(machine.backend.io);
    machine.file = null;

    return .extract_tags;
}

fn stateExtractTags(machine: *ParsingMachine) BackendError!ParsingState {
    const index_bytes = machine.index_bytes orelse return machine.stateFailed(BackendError.InvalidPackage);
    machine.index_bytes = null;
    defer machine.backend.allocator.free(index_bytes);

    const data_block = machine.data_block orelse return machine.stateFailed(BackendError.InvalidPackage);
    machine.data_block = null;
    defer machine.backend.allocator.free(data_block);

    var tag_index: u32 = 0;
    while (tag_index < machine.tag_count) : (tag_index += 1) {
        const entry_bytes = index_bytes[tag_index * 16 ..][0..16];
        const tag_id = std.mem.readInt(u32, entry_bytes[0..4], .big);
        const data_offset = std.mem.readInt(u32, entry_bytes[8..12], .big);

        const tag: RpmTag = @enumFromInt(tag_id);
        switch (tag) {
            .name => machine.raw_meta.name = readString(machine.backend.allocator, data_block, data_offset) catch return machine.stateFailed(BackendError.InvalidPackage),
            .version => machine.raw_meta.version = readString(machine.backend.allocator, data_block, data_offset) catch return machine.stateFailed(BackendError.InvalidPackage),
            .release => machine.raw_meta.release = readString(machine.backend.allocator, data_block, data_offset) catch return machine.stateFailed(BackendError.InvalidPackage),
            .summary => machine.raw_meta.summary = readString(machine.backend.allocator, data_block, data_offset) catch return machine.stateFailed(BackendError.InvalidPackage),
            .arch => machine.raw_meta.arch = readString(machine.backend.allocator, data_block, data_offset) catch return machine.stateFailed(BackendError.InvalidPackage),
            .license => machine.raw_meta.license = readString(machine.backend.allocator, data_block, data_offset) catch return machine.stateFailed(BackendError.InvalidPackage),
            .url => machine.raw_meta.url = readString(machine.backend.allocator, data_block, data_offset) catch return machine.stateFailed(BackendError.InvalidPackage),
            .packager => machine.raw_meta.packager = readString(machine.backend.allocator, data_block, data_offset) catch return machine.stateFailed(BackendError.InvalidPackage),
            .size => {
                if (data_offset + 4 <= data_block.len) {
                    machine.raw_meta.size = @intCast(std.mem.readInt(i32, data_block[data_offset..][0..4], .big));
                }
            },
            _ => {},
        }
    }

    return .build_meta;
}

fn stateBuildMeta(machine: *ParsingMachine) BackendError!ParsingState {
    var sha256: [32]u8 = undefined;

    _ = std.fmt.hexToBytes(&sha256, machine.backend.data.checksum) catch return machine.stateFailed(BackendError.InvalidPackage);

    const raw_version_str = machine.raw_meta.version orelse return machine.stateFailed(BackendError.MetadataNotFound);
    defer machine.backend.allocator.free(raw_version_str);
    machine.raw_meta.version = null;

    const parsed_version = parseVersion(machine.backend.allocator, raw_version_str, machine.raw_meta.release) catch return machine.stateFailed(BackendError.InvalidPackage);
    errdefer parsed_version.deinit(machine.backend.allocator);

    const package_name = machine.raw_meta.name orelse return machine.stateFailed(BackendError.MetadataNotFound);
    machine.raw_meta.name = null;
    errdefer machine.backend.allocator.free(package_name);

    const package_arch = machine.raw_meta.arch orelse return machine.stateFailed(BackendError.MetadataNotFound);
    machine.raw_meta.arch = null;
    errdefer machine.backend.allocator.free(package_arch);

    const package_author = machine.backend.allocator.dupe(u8, machine.raw_meta.packager orelse "") catch return machine.stateFailed(BackendError.OutOfMemory);
    errdefer machine.backend.allocator.free(package_author);

    const package_description = machine.backend.allocator.dupe(u8, machine.raw_meta.summary orelse "") catch return machine.stateFailed(BackendError.OutOfMemory);
    errdefer machine.backend.allocator.free(package_description);

    const package_license = machine.backend.allocator.dupe(u8, machine.raw_meta.license orelse "") catch return machine.stateFailed(BackendError.OutOfMemory);
    errdefer machine.backend.allocator.free(package_license);

    const package_url = machine.backend.allocator.dupe(u8, machine.raw_meta.url orelse "") catch return machine.stateFailed(BackendError.OutOfMemory);
    errdefer machine.backend.allocator.free(package_url);

    const package_packager = machine.backend.allocator.dupe(u8, machine.raw_meta.packager orelse "") catch return machine.stateFailed(BackendError.OutOfMemory);
    errdefer machine.backend.allocator.free(package_packager);

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
