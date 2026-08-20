// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: GPL-3.0-only

use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::Args as ClapArgs;

use self::splice::{find_marked_files, splice};
use self::tree::TreeRenderer;

use crate::error::XtaskError;

pub(crate) mod splice;
mod tree;
mod walk;

#[derive(ClapArgs)]
pub struct Args {
    /// Don't write anything; exit non-zero if any tracked file would change
    #[arg(long = "check")]
    pub check_only: bool,
    /// How many directory levels deep to render
    #[arg(long, default_value_t = 2)]
    pub depth: usize,
}

pub fn run(args: Args) -> Result<ExitCode, XtaskError> {
    let repo_root = repo_root()?;
    let tree_text = TreeRenderer::render(&repo_root, args.depth)?;

    let targets = find_marked_files(&repo_root)?;
    if targets.is_empty() {
        return Err(XtaskError::NoMarkedFiles(repo_root));
    }

    let mut any_stale = false;
    for path in targets {
        let original = fs::read_to_string(&path)?;
        let updated = splice(&original, &tree_text).map_err(|error| XtaskError::Splice {
            path: path.clone(),
            source: Box::new(error),
        })?;

        if original == updated {
            continue;
        }

        any_stale = true;
        let rel = path.strip_prefix(&repo_root).unwrap_or(&path);

        if args.check_only {
            println!("stale: {}", rel.display());
        } else {
            fs::write(&path, updated)?;
            println!("updated: {}", rel.display());
        }
    }

    if args.check_only {
        if any_stale {
            eprintln!("repo tree in docs is out of date — run `cargo xtask gen-tree`");
            return Ok(ExitCode::FAILURE);
        }
        println!("repo tree in docs is up to date");
    } else if !any_stale {
        println!("repo tree in docs was already up to date");
    }

    Ok(ExitCode::SUCCESS)
}

fn repo_root() -> Result<PathBuf, XtaskError> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    manifest_dir
        .parent()
        .map(Path::to_path_buf)
        .ok_or(XtaskError::RepoRootNotFound)
}
