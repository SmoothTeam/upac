// ── Imports ───────────────────────────────────────────────────────────────────
const std = @import("std");

// ── Constants ─────────────────────────────────────────────────────────────────
const LOCK_FILE_PATH: [:0]const u8 = "/run/lock/upac.lock";

// ── Errors ────────────────────────────────────────────────────────────────────
pub const LockError = error{
    WouldBlock,
    LockFailed,
};

// ── Lock ──────────────────────────────────────────────────────────────────────
pub const Lock = struct {
    lock_file_fd: std.posix.fd_t,

    pub fn acquire() LockError!Lock {
        const lock_file_fd = std.posix.open(
            LOCK_FILE_PATH,
            .{ .ACCMODE = .WRONLY, .CREAT = true },
            0o644,
        ) catch return LockError.LockFailed;

        std.posix.flock(lock_file_fd, std.posix.LOCK.EX | std.posix.LOCK.NB) catch |err| {
            std.posix.close(lock_file_fd);
            return switch (err) {
                error.WouldBlock => LockError.WouldBlock,
                else => LockError.LockFailed,
            };
        };

        return .{ .lock_file_fd = lock_file_fd };
    }

    pub fn release(self: Lock) void {
        std.posix.flock(self.lock_file_fd, std.posix.LOCK.UN) catch {};
        std.posix.close(self.lock_file_fd);
    }
};
