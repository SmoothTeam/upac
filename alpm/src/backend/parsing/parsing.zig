// ── Imports ───────────────────────────────────────────────────────────────────
const std = backend.std;

const types = @import("upac-backend-types");
const PackageMetaField = types.PackageMetaField;

const meta_fields = @import("upac-meta-fields");

const parseVersion = @import("utils.zig").parseVersion;

const backend = @import("../backend.zig");
const Machine = backend.BackendMachine;
const BackendError = backend.BackendError;
const PackageMeta = backend.PackageMeta;

const package_meta_field_map = blk: {
    const fields = std.meta.fields(@TypeOf(meta_fields));
    var kvs: [fields.len]struct { []const u8, PackageMetaField } = undefined;
    for (fields, 0..) |field, i| {
        kvs[i] = .{
            @field(meta_fields, field.name),
            @field(PackageMetaField, field.name),
        };
    }
    break :blk std.StaticStringMap(PackageMetaField).initComptime(kvs);
};

// ── ParsingState ──────────────────────────────────────────────────────────────
const ParsingState = enum {
    parse_pkginfo,
    build_meta,
    cleanup_junk,
    done,
};

// ── RawMeta ───────────────────────────────────────────────────────────────────
const RawMeta = struct {
    name: ?[]const u8 = null,
    version_str: ?[]const u8 = null,
    arch: ?[]const u8 = null,
    description: ?[]const u8 = null,
    url: ?[]const u8 = null,
    maintainer: ?[]const u8 = null,
    license: ?[]const u8 = null,
    installed_size: u64 = 0,

    fn deinit(self: *RawMeta, allocator: std.mem.Allocator) void {
        if (self.name) |value| allocator.free(value);
        if (self.version_str) |value| allocator.free(value);
        if (self.arch) |value| allocator.free(value);
        if (self.description) |value| allocator.free(value);
        if (self.url) |value| allocator.free(value);
        if (self.maintainer) |value| allocator.free(value);
        if (self.license) |value| allocator.free(value);
    }
};

// ── ParsingMachine ────────────────────────────────────────────────────────────
const ParsingMachine = struct {
    backend: *Machine,
    raw_meta: RawMeta = .{},

    fn stateFailed(self: *ParsingMachine, err: BackendError) BackendError {
        self.raw_meta.deinit(self.backend.allocator);

        return err;
    }
};

// ── Trampoline ────────────────────────────────────────────────────────────────
pub fn run(machine: *Machine) BackendError!void {
    var parsing = ParsingMachine{ .backend = machine };

    var state = ParsingState.parse_pkginfo;
    while (state != .done) {
        if (machine.data.cancel_token.isCancelled()) return parsing.stateFailed(BackendError.Cancelled);
        state = switch (state) {
            .parse_pkginfo => try stateParsePkginfo(&parsing),
            .build_meta => try stateBuildMeta(&parsing),
            .cleanup_junk => stateCleanupJunk(&parsing),
            .done => unreachable,
        };
    }
}

// ── States ────────────────────────────────────────────────────────────────────
fn stateParsePkginfo(machine: *ParsingMachine) BackendError!ParsingState {
    const pkginfo_content = machine.backend.package_info orelse return machine.stateFailed(BackendError.MetadataNotFound);
    defer {
        machine.backend.allocator.free(pkginfo_content);
        machine.backend.package_info = null;
    }

    var lines = std.mem.splitScalar(u8, pkginfo_content, '\n');
    while (lines.next()) |line| {
        const trimmed_line = std.mem.trim(u8, line, " \t\r");
        if (trimmed_line.len == 0 or trimmed_line[0] == '#') continue;

        const separator_index = std.mem.indexOf(u8, trimmed_line, " = ") orelse continue;

        const key = std.mem.trim(u8, trimmed_line[0..separator_index], " \t");
        const value = std.mem.trim(u8, trimmed_line[separator_index + 3 ..], " \t");

        const field = package_meta_field_map.get(key) orelse continue;
        switch (field) {
            .Package => machine.raw_meta.name = machine.backend.allocator.dupe(u8, value) catch return machine.stateFailed(BackendError.OutOfMemory),
            .Version => machine.raw_meta.version_str = machine.backend.allocator.dupe(u8, value) catch return machine.stateFailed(BackendError.OutOfMemory),
            .@"Installed-Size" => machine.raw_meta.installed_size = std.fmt.parseInt(u64, value, 10) catch 0,
            .Architecture => machine.raw_meta.arch = machine.backend.allocator.dupe(u8, value) catch return machine.stateFailed(BackendError.OutOfMemory),
            .Description => machine.raw_meta.description = machine.backend.allocator.dupe(u8, value) catch return machine.stateFailed(BackendError.OutOfMemory),
            .License => machine.raw_meta.license = machine.backend.allocator.dupe(u8, value) catch return machine.stateFailed(BackendError.OutOfMemory),
            .Homepage => machine.raw_meta.url = machine.backend.allocator.dupe(u8, value) catch return machine.stateFailed(BackendError.OutOfMemory),
            .Maintainer => machine.raw_meta.maintainer = machine.backend.allocator.dupe(u8, value) catch return machine.stateFailed(BackendError.OutOfMemory),
        }
    }

    return .build_meta;
}

fn stateBuildMeta(machine: *ParsingMachine) BackendError!ParsingState {
    var sha256: [32]u8 = undefined;

    std.fmt.hexToBytes(&sha256, machine.backend.data.checksum) catch return machine.stateFailed(BackendError.InvalidPackage);

    const raw_version_str = machine.raw_meta.version_str orelse return machine.stateFailed(BackendError.MetadataNotFound);
    defer machine.backend.allocator.free(raw_version_str);
    machine.raw_meta.version_str = null;

    const parsed_version = parseVersion(machine.backend.allocator, raw_version_str, '-') catch return machine.stateFailed(BackendError.InvalidPackage);
    errdefer parsed_version.deinit(machine.backend.allocator);

    const package_name = machine.raw_meta.name orelse return machine.stateFailed(BackendError.MetadataNotFound);
    machine.raw_meta.name = null;
    errdefer machine.backend.allocator.free(package_name);

    const arch = machine.raw_meta.arch orelse machine.backend.allocator.dupe(u8, "any") catch return machine.stateFailed(BackendError.OutOfMemory);
    machine.raw_meta.arch = null;
    errdefer machine.backend.allocator.free(arch);

    const maintainer = machine.raw_meta.maintainer orelse machine.backend.allocator.dupe(u8, "") catch return machine.stateFailed(BackendError.OutOfMemory);
    machine.raw_meta.maintainer = null;
    errdefer machine.backend.allocator.free(maintainer);

    const description = machine.raw_meta.description orelse machine.backend.allocator.dupe(u8, "") catch return machine.stateFailed(BackendError.OutOfMemory);
    machine.raw_meta.description = null;
    errdefer machine.backend.allocator.free(description);

    const license = machine.raw_meta.license;
    machine.raw_meta.license = null;

    const url = machine.raw_meta.url;
    machine.raw_meta.url = null;

    machine.backend.meta = PackageMeta{
        .name = package_name,
        .version = parsed_version,
        .arch = arch,
        .arch_sub = null,
        .maintainer = maintainer,
        .description = description,
        .license = license,
        .url = url,
        .sha256 = sha256,
        .installed_size = machine.raw.installed_size,
    };

    return .cleanup_junk;
}

fn stateCleanupJunk(machine: *ParsingMachine) ParsingState {
    const temp_package_path = machine.backend.temp_package_path orelse return .done;

    var temp_dir = std.Io.Dir.openDirAbsolute(machine.backend.io, temp_package_path, .{}) catch return .done;
    defer temp_dir.close(machine.backend.io);

    const junk_filenames = [_][]const u8{ ".BUILDINFO", ".MTREE", ".INSTALL", ".CHANGELOG" };
    for (junk_filenames) |junk_filename| temp_dir.deleteFile(machine.backend.io, junk_filename) catch {};

    return .done;
}
