// ── Imports ───────────────────────────────────────────────────────────────────
const std = @import("std");

// ── Types ─────────────────────────────────────────────────────────────────────
pub const Column = struct {
    name: []const u8,
    type: []const u8,
    primary_key: bool = false,
    not_null: bool = false,
    unique: bool = false,
    references: ?[]const u8 = null,
    default: ?std.json.Value = null,
    @"enum": ?[]std.json.Value = null,
};

pub const Index = struct {
    name: []const u8,
    unique: bool = false,
    columns: [][]const u8,
    where: ?[]const u8 = null,
};

pub const TableSchema = struct {
    table: []const u8,
    columns: []Column,
    primary_key: ?[][]const u8 = null,
    unique: ?[][][]const u8 = null,
    checks: ?[][]const u8 = null,
    indexes: ?[]Index = null,
    seed: ?[]std.json.Value = null,

    // ── SQL generation ────────────────────────────────────────────────────────
    pub fn createSql(self: TableSchema, allocator: std.mem.Allocator) ![:0]u8 {
        var sql_request = std.ArrayList(u8).empty;
        defer sql_request.deinit(allocator);

        var parts = std.ArrayList([]u8).empty;
        defer {
            for (parts.items) |line| allocator.free(line);
            parts.deinit(allocator);
        }

        try appendColumnParts(self, allocator, &parts);
        if (self.primary_key) |key| try appendPrimaryKeyPart(key, allocator, &parts);
        try appendCheckParts(self, allocator, &parts);

        try sql_request.appendSlice(allocator, "CREATE TABLE IF NOT EXISTS ");
        try sql_request.appendSlice(allocator, self.table);
        try sql_request.appendSlice(allocator, " (\n");
        for (parts.items, 0..) |part, part_index| {
            try sql_request.appendSlice(allocator, part);
            if (part_index < parts.items.len - 1) try sql_request.appendSlice(allocator, ",");
            try sql_request.appendSlice(allocator, "\n");
        }
        try sql_request.appendSlice(allocator, ")");

        return sql_request.toOwnedSliceSentinel(allocator, 0);
    }

    pub fn uniqueSqls(self: TableSchema, allocator: std.mem.Allocator) ![][:0]u8 {
        var sql_request_unique_buf = std.ArrayList(u8).empty;
        defer sql_request_unique_buf.deinit(allocator);

        const groups = self.unique orelse return &.{};

        const result_sql_unique_request = try allocator.alloc([:0]u8, groups.len);
        errdefer {
            for (result_sql_unique_request) |line| allocator.free(line);
            allocator.free(result_sql_unique_request);
        }

        for (groups, 0..) |group, group_index| {
            try sql_request_unique_buf.appendSlice(allocator, "CREATE UNIQUE INDEX IF NOT EXISTS uq_");
            try sql_request_unique_buf.appendSlice(allocator, self.table);
            for (group) |col| {
                try sql_request_unique_buf.appendSlice(allocator, "_");
                try sql_request_unique_buf.appendSlice(allocator, col);
            }
            try sql_request_unique_buf.appendSlice(allocator, " ON ");
            try sql_request_unique_buf.appendSlice(allocator, self.table);
            try sql_request_unique_buf.appendSlice(allocator, "(");
            for (group, 0..) |col, ci| {
                if (ci > 0) try sql_request_unique_buf.appendSlice(allocator, ", ");
                try sql_request_unique_buf.appendSlice(allocator, col);
            }
            try sql_request_unique_buf.appendSlice(allocator, ")");

            result_sql_unique_request[group_index] = try sql_request_unique_buf.toOwnedSliceSentinel(allocator, 0);
        }

        return result_sql_unique_request;
    }

    pub fn indexSqls(self: TableSchema, allocator: std.mem.Allocator) ![][:0]u8 {
        var sql_request_index_buf = std.ArrayList(u8).empty;
        defer sql_request_index_buf.deinit(allocator);

        const indexes = self.indexes orelse return &.{};

        const result_sql_index_request = try allocator.alloc([:0]u8, indexes.len);
        errdefer {
            for (result_sql_index_request) |string| allocator.free(string);
            allocator.free(result_sql_index_request);
        }

        for (indexes, 0..) |index_struct, index_index| {
            try sql_request_index_buf.appendSlice(allocator, "CREATE ");
            try sql_request_index_buf.appendSlice(allocator, if (index_struct.unique) "UNIQUE " else "");
            try sql_request_index_buf.appendSlice(allocator, "INDEX IF NOT EXISTS ");
            try sql_request_index_buf.appendSlice(allocator, index_struct.name);
            try sql_request_index_buf.appendSlice(allocator, " ON ");
            try sql_request_index_buf.appendSlice(allocator, self.table);
            try sql_request_index_buf.appendSlice(allocator, "(");
            for (index_struct.columns, 0..) |col, col_index| {
                if (col_index > 0) try sql_request_index_buf.appendSlice(allocator, ", ");
                try sql_request_index_buf.appendSlice(allocator, col);
            }
            try sql_request_index_buf.appendSlice(allocator, ")");
            if (index_struct.where) |line| {
                try sql_request_index_buf.appendSlice(allocator, " WHERE ");
                try sql_request_index_buf.appendSlice(allocator, line);
            }

            result_sql_index_request[index_index] = try sql_request_index_buf.toOwnedSliceSentinel(allocator, 0);
        }

        return result_sql_index_request;
    }

    pub fn seedSql(self: TableSchema, allocator: std.mem.Allocator) !?[:0]u8 {
        const seed = self.seed orelse return null;
        if (seed.len == 0) return null;

        var seed_cols = std.ArrayList([]const u8).empty;
        defer seed_cols.deinit(allocator);

        for (self.columns) |col| {
            if (col.primary_key) continue;
            try seed_cols.append(allocator, col.name);
        }

        var out = std.ArrayList(u8).empty;
        defer out.deinit(allocator);

        try out.appendSlice(allocator, "INSERT OR IGNORE INTO ");
        try out.appendSlice(allocator, self.table);
        try out.appendSlice(allocator, " (");
        for (seed_cols.items, 0..) |col, col_index| {
            if (col_index > 0) try out.appendSlice(allocator, ", ");
            try out.appendSlice(allocator, col);
        }
        try out.appendSlice(allocator, ") VALUES\n");

        for (seed, 0..) |entry, ei| {
            const obj = switch (entry) {
                .object => |o| o,
                else => continue,
            };

            try out.appendSlice(allocator, "    (");
            for (seed_cols.items, 0..) |col, col_index| {
                if (col_index > 0) try out.appendSlice(allocator, ", ");
                const val = obj.get(col) orelse std.json.Value.null;
                try writeJsonValue(&out, allocator, val);
            }
            try out.appendSlice(allocator, ")");
            if (ei < seed.len - 1) try out.appendSlice(allocator, ",");
            try out.appendSlice(allocator, "\n");
        }

        return out.toOwnedSliceSentinel(allocator, 0);
    }

    pub fn column(self: TableSchema, name: []const u8) ?Column {
        for (self.columns) |col| {
            if (std.mem.eql(u8, col.name, name)) return col;
        }
        return null;
    }
};

// ── Registry ──────────────────────────────────────────────────────────────────
const TABLE_FILES = [_][]const u8{
    "architectures.json",
    "packages.json",
    "files.json",
    "categories.json",
    "package_categories.json",
    "dependencies.json",
};

pub const Registry = struct {
    allocator: std.mem.Allocator,
    tables: std.StringHashMap(TableSchema),
    parsed: []std.json.Parsed(TableSchema),

    pub fn load(schema_dir: []const u8, io: std.Io, allocator: std.mem.Allocator) !Registry {
        var tables = std.StringHashMap(TableSchema).init(allocator);
        errdefer tables.deinit();

        const parsed_json = try allocator.alloc(std.json.Parsed(TableSchema), TABLE_FILES.len);

        var loaded: usize = 0;
        errdefer {
            for (parsed_json[0..loaded]) |*parsed| parsed.deinit();
            allocator.free(parsed_json);
        }

        for (TABLE_FILES) |filename| {
            const file_path = try std.fs.path.joinZ(allocator, &.{ schema_dir, filename });
            defer allocator.free(file_path);

            const content = try std.Io.Dir.cwd().readFileAlloc(io, file_path, allocator, .limited(64 * 1024));
            defer allocator.free(content);

            const parsed = try std.json.parseFromSlice(TableSchema, allocator, content, .{ .ignore_unknown_fields = true });
            parsed_json[loaded] = parsed;
            loaded += 1;

            try tables.put(parsed.value.table, parsed.value);
        }

        return .{ .allocator = allocator, .tables = tables, .parsed = parsed_json };
    }

    pub fn deinit(self: *Registry) void {
        for (self.parsed) |*parsed| parsed.deinit();
        self.allocator.free(self.parsed);
        self.tables.deinit();
    }

    pub fn get(self: Registry, table_name: []const u8) ?TableSchema {
        return self.tables.get(table_name);
    }
};

// ── Helpers ───────────────────────────────────────────────────────────────────
fn appendColumnParts(self: TableSchema, allocator: std.mem.Allocator, parts: *std.ArrayList([]u8)) !void {
    var buf = std.ArrayList(u8).empty;
    defer buf.deinit(allocator);
    for (self.columns) |col| {
        try buf.appendSlice(allocator, "    ");
        try buf.appendSlice(allocator, col.name);
        try buf.appendSlice(allocator, " ");
        try buf.appendSlice(allocator, col.type);
        if (col.primary_key and self.primary_key == null) try buf.appendSlice(allocator, " PRIMARY KEY");
        if (col.not_null) try buf.appendSlice(allocator, " NOT NULL");
        if (col.unique) try buf.appendSlice(allocator, " UNIQUE");
        if (col.default) |default| try writeDefault(&buf, allocator, default);
        if (col.@"enum") |enum_value| try writeEnumCheck(&buf, allocator, col.name, enum_value);
        if (col.references) |references| {
            try buf.appendSlice(allocator, " REFERENCES ");
            try buf.appendSlice(allocator, references);
        }

        try parts.append(allocator, try buf.toOwnedSlice(allocator));
    }
}

fn appendPrimaryKeyPart(key: [][]const u8, allocator: std.mem.Allocator, parts: *std.ArrayList([]u8)) !void {
    var buf = std.ArrayList(u8).empty;
    defer buf.deinit(allocator);
    try buf.appendSlice(allocator, "    PRIMARY KEY (");
    for (key, 0..) |slice, slice_index| {
        if (slice_index > 0) try buf.appendSlice(allocator, ", ");
        try buf.appendSlice(allocator, slice);
    }
    try buf.appendSlice(allocator, ")");

    try parts.append(allocator, try buf.toOwnedSlice(allocator));
}

fn appendCheckParts(self: TableSchema, allocator: std.mem.Allocator, parts: *std.ArrayList([]u8)) !void {
    const checks = self.checks orelse return;
    for (checks) |check| {
        const string = try std.fmt.allocPrint(allocator, "    CHECK({s})", .{check});
        try parts.append(allocator, string);
    }
}

fn writeDefault(buf: *std.ArrayList(u8), allocator: std.mem.Allocator, val: std.json.Value) !void {
    switch (val) {
        .string => |string| {
            try buf.appendSlice(allocator, " DEFAULT '");
            try buf.appendSlice(allocator, string);
            try buf.appendSlice(allocator, "'");
        },
        .integer => |int| {
            var tmp: [32]u8 = undefined;
            const str = std.fmt.bufPrint(&tmp, " DEFAULT {d}", .{int}) catch unreachable;
            try buf.appendSlice(allocator, str);
        },
        .float => |float| {
            var tmp: [64]u8 = undefined;
            const str = std.fmt.bufPrint(&tmp, " DEFAULT {d}", .{float}) catch unreachable;
            try buf.appendSlice(allocator, str);
        },
        .null => try buf.appendSlice(allocator, " DEFAULT NULL"),
        else => {},
    }
}

fn writeEnumCheck(buf: *std.ArrayList(u8), allocator: std.mem.Allocator, col_name: []const u8, values: []std.json.Value) !void {
    try buf.appendSlice(allocator, " CHECK(");
    try buf.appendSlice(allocator, col_name);
    try buf.appendSlice(allocator, " IN (");
    for (values, 0..) |val, value_index| {
        if (value_index > 0) try buf.appendSlice(allocator, ", ");
        try writeJsonValue(buf, allocator, val);
    }
    try buf.appendSlice(allocator, "))");
}

fn writeJsonValue(buf: *std.ArrayList(u8), allocator: std.mem.Allocator, val: std.json.Value) !void {
    switch (val) {
        .string => |string| {
            try buf.appendSlice(allocator, "'");
            try buf.appendSlice(allocator, string);
            try buf.appendSlice(allocator, "'");
        },
        .integer => |int| {
            var tmp: [32]u8 = undefined;
            const str = std.fmt.bufPrint(&tmp, "{d}", .{int}) catch unreachable;
            try buf.appendSlice(allocator, str);
        },
        .float => |float| {
            var tmp: [64]u8 = undefined;
            const str = std.fmt.bufPrint(&tmp, "{d}", .{float}) catch unreachable;
            try buf.appendSlice(allocator, str);
        },
        .bool => |bool_value| try buf.appendSlice(allocator, if (bool_value) "1" else "0"),
        .null => try buf.appendSlice(allocator, "NULL"),
        else => try buf.appendSlice(allocator, "NULL"),
    }
}
