// SPDX-FileCopyrightText: 2026 2026 JustPav
// SPDX-FileCopyrightText: 2026 JustPav
//
// SPDX-License-Identifier: LGPL-3.0-or-later

// ── Imports ───────────────────────────────────────────────────────────────────
const std = @import("std");

const BackendError = @import("upac-backend-types").BackendError;
const Version = @import("upac-backend-types").Version;

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

// Parses a deb version string: [epoch:]upstream[-revision][~pre]
pub fn parseVersion(allocator: std.mem.Allocator, version_str: []const u8) BackendError!Version {
    var remaining = version_str;

    var epoch: u32 = 0;
    if (std.mem.indexOf(u8, remaining, ":")) |colon_idx| {
        epoch = std.fmt.parseInt(u32, remaining[0..colon_idx], 10) catch 0;
        remaining = remaining[colon_idx + 1 ..];
    }

    var release: u32 = 0;
    if (std.mem.lastIndexOf(u8, remaining, "-")) |dash_idx| {
        release = std.fmt.parseInt(u32, remaining[dash_idx + 1 ..], 10) catch 0;
        remaining = remaining[0..dash_idx];
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
    while (iter.next()) |part_str| {
        if (part_str.len == 0) continue;
        const part = std.fmt.parseInt(u32, part_str, 10) catch return BackendError.InvalidPackage;
        parts_list.append(allocator, part) catch return BackendError.AllocZFailed;
    }

    if (parts_list.items.len == 0) return BackendError.InvalidPackage;

    const parts_owned = parts_list.toOwnedSlice(allocator) catch return BackendError.AllocZFailed;

    return Version{
        .epoch = epoch,
        .parts = parts_owned,
        .pre = pre,
        .release = release,
    };
}
