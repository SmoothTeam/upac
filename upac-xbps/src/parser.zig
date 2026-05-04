// ── Imports ─────────────────────────────────────────────────────────────────────
const backend = @import("backend.zig");
const std = backend.std;

// ── Public types ─────────────────────────────────────────────────────────────
// Contains metadata extracted from props.plist
pub const XbpsProps = struct {
    name: ?[]const u8 = null,
    version: ?[]const u8 = null,
    architecture: ?[]const u8 = null,
    short_desc: ?[]const u8 = null,
    maintainer: ?[]const u8 = null,
    license: ?[]const u8 = null,
    homepage: ?[]const u8 = null,
    installed_size: u32 = 0,

    pub fn deinit(self: *const XbpsProps, allocator: std.mem.Allocator) void {
        if (self.name) |variant| allocator.free(variant);
        if (self.version) |variant| allocator.free(variant);
        if (self.architecture) |variant| allocator.free(variant);
        if (self.short_desc) |variant| allocator.free(variant);
        if (self.maintainer) |variant| allocator.free(variant);
        if (self.license) |variant| allocator.free(variant);
        if (self.homepage) |variant| allocator.free(variant);
    }
};

// ── Parser ────────────────────────────────────────────────────────────────────
// Parses an Apple XML plist (props.plist from .xbps archive).
// Looks for <key>K</key> followed by <string>V</string> or <integer>N</integer>.
pub fn parse(allocator: std.mem.Allocator, content: []const u8) !XbpsProps {
    var props = XbpsProps{};
    errdefer props.deinit(allocator);

    var pos: usize = 0;
    while (pos < content.len) {
        // Find next <key>
        const key_open = std.mem.indexOfPos(u8, content, pos, "<key>") orelse break;
        const key_start = key_open + "<key>".len;
        const key_close = std.mem.indexOfPos(u8, content, key_start, "</key>") orelse break;
        const key = content[key_start..key_close];
        pos = key_close + "</key>".len;

        // Skip whitespace between </key> and the next tag
        while (pos < content.len and (content[pos] == ' ' or content[pos] == '\t' or
            content[pos] == '\n' or content[pos] == '\r')) pos += 1;

        if (std.mem.startsWith(u8, content[pos..], "<string>")) {
            const val_start = pos + "<string>".len;
            const val_close = std.mem.indexOfPos(u8, content, val_start, "</string>") orelse break;
            const value = content[val_start..val_close];
            pos = val_close + "</string>".len;

            if (std.mem.eql(u8, key, "pkgname")) {
                props.name = try decodeXmlEntities(allocator, value);
            } else if (std.mem.eql(u8, key, "version")) {
                props.version = try decodeXmlEntities(allocator, value);
            } else if (std.mem.eql(u8, key, "architecture")) {
                props.architecture = try decodeXmlEntities(allocator, value);
            } else if (std.mem.eql(u8, key, "short_desc")) {
                props.short_desc = try decodeXmlEntities(allocator, value);
            } else if (std.mem.eql(u8, key, "maintainer")) {
                props.maintainer = try decodeXmlEntities(allocator, value);
            } else if (std.mem.eql(u8, key, "license")) {
                props.license = try decodeXmlEntities(allocator, value);
            } else if (std.mem.eql(u8, key, "homepage")) {
                props.homepage = try decodeXmlEntities(allocator, value);
            }
        } else if (std.mem.startsWith(u8, content[pos..], "<integer>")) {
            const val_start = pos + "<integer>".len;
            const val_close = std.mem.indexOfPos(u8, content, val_start, "</integer>") orelse break;
            const value = content[val_start..val_close];
            pos = val_close + "</integer>".len;

            if (std.mem.eql(u8, key, "installed_size")) {
                props.installed_size = std.fmt.parseInt(u32, value, 10) catch 0;
            }
        } else {
            // Unknown tag type — skip to next <key>
            pos += 1;
        }
    }

    return props;
}

// ── Helpers ───────────────────────────────────────────────────────────────────
fn decodeXmlEntities(allocator: std.mem.Allocator, input: []const u8) ![]const u8 {
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
