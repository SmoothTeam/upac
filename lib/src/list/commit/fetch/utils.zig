const std = @import("std");

const ffi = @import("upac-ffi");
const CSlice = ffi.CSlice;
const CCommitEntry = ffi.CCommitEntry;

const CommitEntry = @import("../commit.zig").CommitEntry;

pub fn dupeRow(checksum: [*c]u8, subject: []const u8, allocator: std.mem.Allocator) !CommitEntry {
    const checksum_dupe = try allocator.dupe(u8, std.mem.span(checksum));
    errdefer allocator.free(checksum_dupe);

    const subject_dupe = try allocator.dupe(u8, subject);

    return .{ .checksum = checksum_dupe, .subject = subject_dupe };
}

pub fn convertCommitEntry(row: CommitEntry) CCommitEntry {
    return .{
        .checksum = CSlice.fromSlice(row.checksum),
        .subject = CSlice.fromSlice(row.subject),
    };
}
