// SPDX-FileCopyrightText: 2026 JustPav
//
// SPDX-License-Identifier: GPL-3.0-only

use std::fs;
use std::path::{Path, PathBuf};

use super::walk::read_dir_filtered;

use crate::error::XtaskError;

pub(crate) const MARKER_START: &str = "<!-- tree:start -->";
pub(crate) const MARKER_END: &str = "<!-- tree:end -->";

pub fn find_marked_files(root: &Path) -> Result<Vec<PathBuf>, XtaskError> {
    let mut found = Vec::new();
    walk(root, &mut found)?;

    Ok(found)
}

fn walk(dir: &Path, found: &mut Vec<PathBuf>) -> Result<(), XtaskError> {
    for entry in read_dir_filtered(dir)? {
        let path = entry.path();

        if path.is_dir() {
            walk(&path, found)?;
            continue;
        }

        // Only markdown is a splice target — this also keeps xtask's own
        // source (which necessarily mentions the marker strings) out of it.
        let is_markdown = path.extension().is_some_and(|ext| ext.eq_ignore_ascii_case("md"));
        if !is_markdown {
            continue;
        }

        let content = fs::read_to_string(&path)?;
        if content.contains(MARKER_START) && content.contains(MARKER_END) {
            found.push(path);
        }
    }

    Ok(())
}

pub fn splice(original: &str, tree_text: &str) -> Result<String, XtaskError> {
    let start = original.find(MARKER_START).ok_or(XtaskError::MissingStartMarker)?;
    let end = original.find(MARKER_END).ok_or(XtaskError::MissingEndMarker)?;

    if end < start {
        return Err(XtaskError::EndBeforeStart);
    }
    if original[start + MARKER_START.len()..].matches(MARKER_START).count() > 0 {
        return Err(XtaskError::DuplicateStartMarker);
    }

    let end = end + MARKER_END.len();
    let block = format!("{MARKER_START}\n```text\n{tree_text}\n```\n{MARKER_END}");

    Ok(format!("{}{}{}", &original[..start], block, &original[end..]))
}
