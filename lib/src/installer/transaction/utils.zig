const std = @import("std");

const c_libs = @import("c-libs");

const InstallerError = @import("../installer.zig").InstallerError;

const TransactionMachine = @import("transaction.zig").TransactionMachine;

pub fn formatVersion(version: @import("upac-types").Version, writer: *std.Io.Writer) !void {
    if (version.epoch > 0) try writer.print("{d}:", .{version.epoch});
    for (version.parts, 0..) |part, index| {
        if (index > 0) try writer.print(".", .{});
        try writer.print("{d}", .{part});
    }
    if (version.pre) |pre| try writer.print("~{s}", .{pre});
    if (version.release > 1) try writer.print("-{d}", .{version.release});
}
