// ── Imports ─────────────────────────────────────────────────────────────────────
const backend = @import("backend.zig");
const std = backend.std;

// ── Contains RPM magic bytes and header magic bytes ─────────────────────────────────────────────────────────────
const rpm_magic: [4]u8 = .{ 0xED, 0xAB, 0xEE, 0xDB };
const header_magic: [3]u8 = .{ 0x8E, 0xAD, 0xE8 };

// Represents an RPM tag, identified by its numeric tag ID
const RpmTag = enum(u32) {
    name = 1000,
    version = 1001,
    release = 1002,
    summary = 1004,
    description = 1005,
    license = 1014,
    packager = 1015,
    url = 1020,
    arch = 1022,
    size = 1023,
    _,
};

const TagEntry = struct {
    tag: u32,
    tag_type: u32,
    offset: u32,
    count: u32,
};

const SectionHeader = struct { tag_count: u32, data_size: u32 };

// ── Public types ────────────────────────────────────────────────────────────
// Contains metadata extracted from the RPM package header
pub const RpmHeader = struct {
    name: ?[]const u8 = null,
    version: ?[]const u8 = null,
    size: u32 = 0,
    release: ?[]const u8 = null,
    summary: ?[]const u8 = null,
    arch: ?[]const u8 = null,
    license: ?[]const u8 = null,
    url: ?[]const u8 = null,
    packager: ?[]const u8 = null,

    pub fn deinit(self: *RpmHeader, allocator: std.mem.Allocator) void {
        if (self.name) |value| allocator.free(value);
        if (self.version) |value| allocator.free(value);
        if (self.release) |value| allocator.free(value);
        if (self.summary) |value| allocator.free(value);
        if (self.arch) |value| allocator.free(value);
        if (self.license) |value| allocator.free(value);
        if (self.url) |value| allocator.free(value);
        if (self.packager) |value| allocator.free(value);
    }
};

// ── Parser ────────────────────────────────────────────────────────────────────
pub fn parseHeader(allocator: std.mem.Allocator, file: std.fs.File) !RpmHeader {
    try verifyMagic(file);
    try skipLeadSection(file);
    try skipSignatureSection(file);
    return readHeaderSection(allocator, file);
}

// ── Internal functions ────────────────────────────────────────────────────────

// Reads exactly buf.len bytes; returns error.UnexpectedEOF on short read.
fn readExact(file: std.fs.File, buf: []u8) !void {
    var total: usize = 0;
    while (total < buf.len) {
        const n = try file.read(buf[total..]);
        if (n == 0) return error.UnexpectedEOF;
        total += n;
    }
}

fn verifyMagic(file: std.fs.File) !void {
    var magic_buffer: [4]u8 = undefined;
    try readExact(file, &magic_buffer);
    if (!std.mem.eql(u8, &magic_buffer, &rpm_magic)) return error.InvalidRpmMagic;
}

// Skips the obsolete Lead section (96 bytes minus the 4 already read for magic)
fn skipLeadSection(file: std.fs.File) !void {
    try file.seekBy(96 - 4);
}

// Reads the 16-byte section intro; checks magic and returns tag_count + data_size.
fn readSectionHeader(file: std.fs.File, comptime err: anyerror) !SectionHeader {
    var buf: [16]u8 = undefined;
    try readExact(file, &buf);
    if (!std.mem.eql(u8, buf[0..3], &header_magic)) return err;
    return .{
        .tag_count = std.mem.readInt(u32, buf[8..12], .big),
        .data_size = std.mem.readInt(u32, buf[12..16], .big),
    };
}

// Skips the digital signature section, including its 8-byte alignment padding.
fn skipSignatureSection(file: std.fs.File) !void {
    const header = try readSectionHeader(file, error.InvalidSignatureMagic);

    const tags_size: u64 = @as(u64, header.tag_count) * 16;
    const data_size: u64 = header.data_size;
    try file.seekBy(@intCast(tags_size + data_size));

    // Pad to 8-byte boundary (only index+data counts, header is already 8-aligned)
    const remainder = (tags_size + data_size) % 8;
    if (remainder != 0) try file.seekBy(@intCast(8 - remainder));
}

// Reads the main header section, extracting the tag table and data block.
fn readHeaderSection(allocator: std.mem.Allocator, file: std.fs.File) !RpmHeader {
    const header = try readSectionHeader(file, error.InvalidHeaderMagic);

    // Read all tag index entries as a flat byte slice, then parse in-place.
    const index_bytes = try allocator.alloc(u8, @as(usize, header.tag_count) * 16);
    defer allocator.free(index_bytes);
    try readExact(file, index_bytes);

    // Read the data store.
    const data_block = try allocator.alloc(u8, header.data_size);
    defer allocator.free(data_block);
    try readExact(file, data_block);

    var rpm_header = RpmHeader{};
    errdefer rpm_header.deinit(allocator);

    var i: usize = 0;
    while (i < header.tag_count) : (i += 1) {
        const e = index_bytes[i * 16 ..][0..16];
        const tag = std.mem.readInt(u32, e[0..4], .big);
        const offset = std.mem.readInt(u32, e[8..12], .big);

        const rpm_tag = std.meta.intToEnum(RpmTag, tag) catch continue;
        switch (rpm_tag) {
            .name => rpm_header.name = try readString(allocator, data_block, offset),
            .version => rpm_header.version = try readString(allocator, data_block, offset),
            .release => rpm_header.release = try readString(allocator, data_block, offset),
            .summary => rpm_header.summary = try readString(allocator, data_block, offset),
            .arch => rpm_header.arch = try readString(allocator, data_block, offset),
            .license => rpm_header.license = try readString(allocator, data_block, offset),
            .url => rpm_header.url = try readString(allocator, data_block, offset),
            .packager => rpm_header.packager = try readString(allocator, data_block, offset),
            .size => {
                if (offset + 4 <= data_block.len) {
                    rpm_header.size = @intCast(std.mem.readInt(i32, data_block[offset..][0..4], .big));
                }
            },
            else => {},
        }
    }

    return rpm_header;
}

// Reads a null-terminated string from a data block at a specified offset.
fn readString(allocator: std.mem.Allocator, data_block: []const u8, offset: u32) ![]const u8 {
    if (offset >= data_block.len) return error.InvalidTagOffset;
    const start = data_block[offset..];
    const end = std.mem.indexOfScalar(u8, start, 0) orelse return error.UnterminatedString;
    return allocator.dupe(u8, start[0..end]);
}
