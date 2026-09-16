// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: GPL-3.0-only

use std::fs::{self, DirEntry};
use std::path::Path;

use crate::error::XtaskError;

const IGNORED_DIRS: &[&str] = &[".git", "target", "node_modules", ".zig-cache", "zig-out", "zig-pkg"];

pub(super) fn read_dir_filtered(dir: &Path) -> Result<Vec<DirEntry>, XtaskError> {
    let mut entries = Vec::new();

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let name = entry.file_name();

        if IGNORED_DIRS.contains(&name.to_string_lossy().as_ref()) {
            continue;
        }

        entries.push(entry);
    }

    Ok(entries)
}
