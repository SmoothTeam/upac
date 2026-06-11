// ── Imports ─────────────────────────────────────────────────────────────────────
const std = @import("std");

const types = @import("upac-backend-types");
const BackendError = types.BackendError;
const Version = types.Version;

// Parses an XBPS version string: [epoch:]version_release
// version can contain ~ for pre-release: 3.6.2~alpha_1
pub fn parseVersion(allocator: std.mem.Allocator, version_str: []const u8) BackendError!Version {
    var remaining = version_str;

    var epoch: u32 = 0;
    if (std.mem.indexOf(u8, remaining, ":")) |colon_idx| {
        epoch = std.fmt.parseInt(u32, remaining[0..colon_idx], 10) catch 0;
        remaining = remaining[colon_idx + 1 ..];
    }

    var release: u32 = 1;
    if (std.mem.lastIndexOf(u8, remaining, "_")) |underscore_idx| {
        release = std.fmt.parseInt(u32, remaining[underscore_idx + 1 ..], 10) catch 1;
        remaining = remaining[0..underscore_idx];
    }

    var pre: ?[]const u8 = null;
    if (std.mem.indexOf(u8, remaining, "~")) |tilde_idx| {
        pre = allocator.dupe(u8, remaining[tilde_idx + 1 ..]) catch return BackendError.AllocZFailed;
        remaining = remaining[0..tilde_idx];
    }
    errdefer if (pre) |p| allocator.free(p);

    var parts_list = std.ArrayList(u32).empty;
    defer parts_list.deinit(allocator);

    var iter = std.mem.splitScalar(u8, remaining, '.');
    while (iter.next()) |segment| {
        if (segment.len == 0) continue;

        const digits_end = for (segment, 0..) |ch, i| {
            if (ch < '0' or ch > '9') break i;
        } else segment.len;

        if (digits_end == 0) continue;

        const num = std.fmt.parseInt(u32, segment[0..digits_end], 10) catch continue;
        parts_list.append(allocator, num) catch return BackendError.AllocZFailed;

        if (digits_end < segment.len and pre == null) {
            pre = allocator.dupe(u8, segment[digits_end..]) catch return BackendError.AllocZFailed;
        }
    }

    if (parts_list.items.len == 0) return BackendError.InvalidPackage;

    const parts_owned = parts_list.toOwnedSlice(allocator) catch return BackendError.AllocZFailed;
    errdefer allocator.free(parts_owned);

    return Version{ .epoch = epoch, .parts = parts_owned, .pre = pre, .release = release };
}

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
