// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later

// ── Imports ───────────────────────────────────────────────────────────────────
const std = @import("std");

const types = @import("upac-backend-types");
const Version = types.Version;
const BackendError = types.BackendError;

// Parses a version string "epoch:X.Y.Z-release" where release_sep separates the release suffix.
// epoch prefix "N:" is optional. If the release suffix is non-numeric, it becomes the pre-release tag.
pub fn parseVersion(allocator: std.mem.Allocator, version_str: []const u8, release_sep: u8) BackendError!Version {
    var epoch: u32 = 0;
    var release: u32 = 1;
    var pre: ?[]const u8 = null;
    var remaining = version_str;
    var version_parts = std.ArrayList(u32).empty;
    errdefer version_parts.deinit(allocator);

    if (std.mem.indexOf(u8, remaining, ":")) |colon_idx| {
        if (std.fmt.parseInt(u32, remaining[0..colon_idx], 10)) |parsed_epoch| {
            epoch = parsed_epoch;
            remaining = remaining[colon_idx + 1 ..];
        } else |_| {}
    }

    if (std.mem.lastIndexOf(u8, remaining, &[_]u8{release_sep})) |sep_idx| {
        const release_str = remaining[sep_idx + 1 ..];
        if (std.fmt.parseInt(u32, release_str, 10)) |parsed_release| {
            release = parsed_release;
        } else |_| {
            var digit_end: usize = 0;
            while (digit_end < release_str.len and release_str[digit_end] >= '0' and release_str[digit_end] <= '9') : (digit_end += 1) {}

            if (digit_end > 0) release = std.fmt.parseInt(u32, release_str[0..digit_end], 10) catch 1;
            if (release_str.len > 0) pre = allocator.dupe(u8, release_str) catch return BackendError.AllocZFailed;
        }
        remaining = remaining[0..sep_idx];
    }

    var part_iter = std.mem.splitScalar(u8, remaining, '.');
    while (part_iter.next()) |part_str| {
        var digit_end: usize = 0;
        while (digit_end < part_str.len and part_str[digit_end] >= '0' and part_str[digit_end] <= '9') : (digit_end += 1) {}
        const part_value = if (digit_end > 0) std.fmt.parseInt(u32, part_str[0..digit_end], 10) catch 0 else 0;
        version_parts.append(allocator, part_value) catch return BackendError.AllocZFailed;
    }

    if (version_parts.items.len == 0) version_parts.append(allocator, 0) catch return BackendError.AllocZFailed;

    return Version{
        .epoch = epoch,
        .parts = version_parts.toOwnedSlice(allocator) catch return BackendError.AllocZFailed,
        .pre = pre,
        .release = release,
    };
}
