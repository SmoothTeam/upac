// SPDX-FileCopyrightText: 2026 JustPav
//
// SPDX-License-Identifier: GPL-3.0-only

use std::fs;
use std::path::{Path, PathBuf};

use crate::error::XtaskError;

const IGNORED_DIRS: &[&str] = &[".git", "target", "node_modules", ".zig-cache", "zig-out", "zig-pkg"];

pub(super) fn find_rust_files(root: &Path) -> Result<Vec<PathBuf>, XtaskError> {
    find_files(root, |path| path.extension().is_some_and(|extension| extension == "rs"))
}

pub(super) fn find_cargo_toml_files(root: &Path) -> Result<Vec<PathBuf>, XtaskError> {
    find_files(root, |path| path.file_name().is_some_and(|name| name == "Cargo.toml"))
}

pub(super) fn find_plain_toml_files(root: &Path) -> Result<Vec<PathBuf>, XtaskError> {
    find_files(root, |path| {
        path.extension().is_some_and(|extension| extension == "toml")
            && path.file_name().is_some_and(|name| name != "Cargo.toml")
    })
}

fn find_files(root: &Path, predicate: impl Fn(&Path) -> bool + Copy) -> Result<Vec<PathBuf>, XtaskError> {
    let mut files = Vec::new();
    collect(root, predicate, &mut files)?;

    Ok(files)
}

fn collect(dir: &Path, predicate: impl Fn(&Path) -> bool + Copy, files: &mut Vec<PathBuf>) -> Result<(), XtaskError> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let name = entry.file_name();

        if IGNORED_DIRS.contains(&name.to_string_lossy().as_ref()) {
            continue;
        }

        let path = entry.path();
        if entry.file_type()?.is_dir() {
            collect(&path, predicate, files)?;
        } else if predicate(&path) {
            files.push(path);
        }
    }

    Ok(())
}
