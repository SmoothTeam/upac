// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: GPL-3.0-only

//! `cargo xtask vendor-setup-root`
//!
//! Builds `composefs-setup-root` (crates.io, bin-only — no `[lib]` target, so it can't be pulled
//! in as a normal `[dependencies]` entry) via `cargo install --root`, dropping the resulting
//! binary under `target/composefs-setup-root/bin/`, alongside the rest of the workspace's own
//! build output.

use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

use crate::error::XtaskError;

const CRATE_NAME: &str = "composefs-setup-root";

pub fn run() -> Result<ExitCode, XtaskError> {
    let install_root = repo_root()?.join("target").join(CRATE_NAME);

    let status = Command::new("cargo")
        .args(["install", CRATE_NAME, "--root"])
        .arg(&install_root)
        .status()?;

    if !status.success() {
        return Ok(ExitCode::FAILURE);
    }

    println!(
        "{CRATE_NAME} installed at {}",
        install_root.join("bin").join(CRATE_NAME).display()
    );

    Ok(ExitCode::SUCCESS)
}

fn repo_root() -> Result<PathBuf, XtaskError> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    manifest_dir
        .parent()
        .map(Path::to_path_buf)
        .ok_or(XtaskError::RepoRootNotFound)
}
