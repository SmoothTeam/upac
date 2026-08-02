// SPDX-FileCopyrightText: 2026 2026 JustPav
// SPDX-FileCopyrightText: 2026 JustPav
//
// SPDX-License-Identifier: LGPL-3.0-or-later

// ── Imports ───────────────────────────────────────────────────────────────────
const std = @import("std");

const types = @import("upac-backend-types");
const rpm_lead_magic = types.rpm_lead_magic;
const rpm_lead_size = types.rpm_lead_size;

const BackendError = types.BackendError;

const backend = @import("../backend.zig");
const Machine = backend.BackendMachine;

// ── VerifyingState ────────────────────────────────────────────────────────────
const VerifyingState = enum {
    check_file,
    check_temp_dir,
    hash,
    compare,
    validate_lead,
    done,
};

// ── VerifyingMachine ──────────────────────────────────────────────────────────
const VerifyingMachine = struct {
    backend: *Machine,

    file: ?std.Io.File = null,

    digest_checksum: [32]u8 = undefined,

    fn stateFailed(self: *VerifyingMachine, err: BackendError) BackendError {
        if (self.file) |file| {
            file.close(self.backend.io);
            self.file = null;
        }

        return err;
    }
};

// ── Trampoline ────────────────────────────────────────────────────────────────
pub fn run(machine: *Machine) BackendError!void {
    var verifying = VerifyingMachine{ .backend = machine };

    var state = VerifyingState.check_file;
    while (state != .done) {
        if (machine.data.cancel_token.isCancelled()) return verifying.stateFailed(BackendError.Cancelled);
        state = switch (state) {
            .check_file => try stateCheckFile(&verifying),
            .check_temp_dir => try stateCheckTempDir(&verifying),
            .hash => try stateHash(&verifying),
            .compare => try stateCompare(&verifying),
            .validate_lead => try stateValidateLead(&verifying),
            .done => unreachable,
        };
    }
}

// ── States ────────────────────────────────────────────────────────────────────
fn stateCheckFile(machine: *VerifyingMachine) BackendError!VerifyingState {
    const package_path = std.mem.span(machine.backend.data.package_path_c);

    std.Io.Dir.accessAbsolute(machine.backend.io, package_path, .{}) catch return machine.stateFailed(BackendError.ReadFailed);

    return .check_temp_dir;
}

fn stateCheckTempDir(machine: *VerifyingMachine) BackendError!VerifyingState {
    const temp_path = std.mem.span(machine.backend.data.temp_path_c);

    std.Io.Dir.accessAbsolute(machine.backend.io, temp_path, .{}) catch return machine.stateFailed(BackendError.TempDirFailed);

    return .hash;
}

fn stateHash(machine: *VerifyingMachine) BackendError!VerifyingState {
    var package_reader_buf: [65536]u8 = undefined;
    var package_hasher = std.crypto.hash.sha2.Sha256.init(.{});

    const package_path = std.mem.span(machine.backend.data.package_path_c);

    const file = std.Io.Dir.openFileAbsolute(machine.backend.io, package_path, .{}) catch return machine.stateFailed(BackendError.ReadFailed);
    machine.file = file;

    var package_read_bufs_vector = [1][]u8{package_reader_buf[0..]};
    while (true) {
        const bytes_read = file.readStreaming(machine.backend.io, &package_read_bufs_vector) catch |err| {
            if (err == error.EndOfStream) break;
            return machine.stateFailed(BackendError.ReadFailed);
        };

        if (bytes_read == 0) break;

        package_hasher.update(package_reader_buf[0..bytes_read]);
    }

    package_hasher.final(&machine.digest_checksum);

    return .compare;
}

fn stateCompare(machine: *VerifyingMachine) BackendError!VerifyingState {
    var checksum_as_bytes: [std.crypto.hash.sha2.Sha256.digest_length]u8 = undefined;

    _ = std.fmt.hexToBytes(&checksum_as_bytes, machine.backend.data.checksum) catch return machine.stateFailed(BackendError.InvalidPackage);

    if (!std.mem.eql(u8, &machine.digest_checksum, &checksum_as_bytes)) return machine.stateFailed(BackendError.ChecksumMismatch);

    if (machine.file) |file| {
        file.close(machine.backend.io);
        machine.file = null;
    }

    return .validate_lead;
}

fn stateValidateLead(machine: *VerifyingMachine) BackendError!VerifyingState {
    const package_path = std.mem.span(machine.backend.data.package_path_c);

    const file = std.Io.Dir.openFileAbsolute(machine.backend.io, package_path, .{}) catch return machine.stateFailed(BackendError.ReadFailed);
    defer file.close(machine.backend.io);

    var lead_buf: [rpm_lead_size]u8 = undefined;
    var lead_iov = [1][]u8{lead_buf[0..]};
    const bytes_read = file.readStreaming(machine.backend.io, &lead_iov) catch return machine.stateFailed(BackendError.ReadFailed);

    if (bytes_read < rpm_lead_size) return machine.stateFailed(BackendError.InvalidPackage);
    if (!std.mem.eql(u8, lead_buf[0..4], &rpm_lead_magic)) return machine.stateFailed(BackendError.InvalidPackage);
    if (lead_buf[4] < 3) return machine.stateFailed(BackendError.InvalidPackage);

    return .done;
}
