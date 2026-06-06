// ── Imports ───────────────────────────────────────────────────────────────────
const std = @import("std");

pub fn parseLicenseFromCopyright(content: []const u8, allocator: std.mem.Allocator) std.mem.Allocator.Error![]const u8 {
    var lines = std.mem.splitScalar(u8, content, '\n');
    while (lines.next()) |line| {
        const trimmed_line = std.mem.trim(u8, line, " \t\r");
        if (!std.mem.startsWith(u8, trimmed_line, "License:")) continue;
        const value = std.mem.trim(u8, trimmed_line["License:".len..], " \t\r");
        if (value.len == 0) continue;
        return allocator.dupe(u8, value);
    }
    return allocator.dupe(u8, "");
}
