// SPDX-FileCopyrightText: 2026 JustPav
//
// SPDX-License-Identifier: GPL-3.0-only

//! `cargo xtask lint-style`
//!
//! Checks the mechanically-checkable subset of `CONTRIBUTING.md`'s style rules — the ones that
//! are pure syntax/structure, not judgment calls. Prints one line per violation and exits
//! non-zero if any were found.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use crate::error::XtaskError;

mod cargo_toml_dependency_order;
mod cargo_toml_package_order;
mod extern_fn_position;
mod macro_visibility_adjacency;
mod no_pub_use_reexport;
mod toml_config_field_order;
mod violation;
mod walk;

pub fn run() -> Result<ExitCode, XtaskError> {
    let repo_root = repo_root()?;
    let rust_files = walk::find_rust_files(&repo_root)?;
    let cargo_toml_files = walk::find_cargo_toml_files(&repo_root)?;
    let plain_toml_files = walk::find_plain_toml_files(&repo_root)?;

    let mut violations = Vec::new();
    for path in &rust_files {
        let contents = fs::read_to_string(path)?;
        violations.extend(no_pub_use_reexport::check(path, &contents));
        violations.extend(extern_fn_position::check(path, &contents));
        violations.extend(macro_visibility_adjacency::check(path, &contents));
    }
    for path in &cargo_toml_files {
        let contents = fs::read_to_string(path)?;
        violations.extend(cargo_toml_package_order::check(path, &contents));
        violations.extend(cargo_toml_dependency_order::check(path, &contents));
    }
    for path in &plain_toml_files {
        let contents = fs::read_to_string(path)?;
        violations.extend(toml_config_field_order::check(path, &contents));
    }

    if violations.is_empty() {
        println!("lint-style: clean ({} files checked)", rust_files.len());
        return Ok(ExitCode::SUCCESS);
    }

    for violation in &violations {
        println!("{violation}");
    }
    eprintln!("lint-style: {} violation(s)", violations.len());

    Ok(ExitCode::FAILURE)
}

fn repo_root() -> Result<PathBuf, XtaskError> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    manifest_dir.parent().map(Path::to_path_buf).ok_or(XtaskError::RepoRootNotFound)
}
