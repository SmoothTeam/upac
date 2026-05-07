// ── upac deb backend tests ────────────────────────────────────────────────────
// Stub. Real tests will live alongside this file.

const std = @import("std");

test "smoke" {
    try std.testing.expectEqual(@as(i32, 4), 2 + 2);
}
