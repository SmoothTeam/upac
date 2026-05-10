// ── Global constants ─────────────────────────────────────────────────────────
// Hard-coded paths and identifiers shared across the upac core library.
// These exist so the rest of the codebase never sprinkles magic strings like
// "usr" or "usr/share/upac/db" inline.

// The single, atomically-swappable prefix directory. Everything that should be
// part of an atomic upgrade lives under <root>/<PREFIX>/. External entries like
// `/opt` are realised as symlinks pointing inside this prefix (see init).
pub const PREFIX: [:0]const u8 = "usr";

// Path of the upac package database, relative to `root_path`.
// Always read/written as join(root_path, DB_RELATIVE_PATH).
pub const DB_RELATIVE_PATH: []const u8 = PREFIX ++ "/share/upac/db";
