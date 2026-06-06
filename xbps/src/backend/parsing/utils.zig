// ── Imports ─────────────────────────────────────────────────────────────────────
const std = @import("std");

pub fn decodeXmlEntities(allocator: std.mem.Allocator, input: []const u8) ![]const u8 {
    var out = std.ArrayList(u8).empty;
    errdefer out.deinit(allocator);

    var index: usize = 0;
    while (index < input.len) {
        if (input[index] == '&') {
            if (std.mem.startsWith(u8, input[index..], "&lt;")) {
                try out.append(allocator, '<');
                index += "&lt;".len;
            } else if (std.mem.startsWith(u8, input[index..], "&gt;")) {
                try out.append(allocator, '>');
                index += "&gt;".len;
            } else if (std.mem.startsWith(u8, input[index..], "&amp;")) {
                try out.append(allocator, '&');
                index += "&amp;".len;
            } else if (std.mem.startsWith(u8, input[index..], "&apos;")) {
                try out.append(allocator, '\'');
                index += "&apos;".len;
            } else if (std.mem.startsWith(u8, input[index..], "&quot;")) {
                try out.append(allocator, '"');
                index += "&quot;".len;
            } else {
                try out.append(allocator, input[index]);
                index += 1;
            }
        } else {
            try out.append(allocator, input[index]);
            index += 1;
        }
    }

    return out.toOwnedSlice(allocator);
}
