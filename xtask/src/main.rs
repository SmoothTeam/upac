// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: GPL-3.0-only

//! `cargo xtask gen-tree [--check] [--depth N]`
//!
//! Walks the repository from its root and renders an ASCII directory tree
//! (dirs first, alphabetical, Unicode box-drawing characters — matching the
//! style already used in `doc/`). The tree is then spliced into every
//! markdown file that contains a `<!-- tree:start -->` / `<!-- tree:end -->`
//! marker pair, replacing everything between the markers (inclusive) with a
//! freshly generated fenced code block.
//!
//! `--check` doesn't write anything: it exits non-zero if any tracked file
//! would change, which is what CI should call.

use std::process::ExitCode;

use self::error::XtaskError;

mod error;
mod gen_tree;
mod lint_style;

fn main() -> ExitCode {
    match run() {
        Ok(code) => code,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<ExitCode, XtaskError> {
    let mut args = std::env::args().skip(1);
    let command = args.next().ok_or(XtaskError::MissingCommand)?;

    match command.as_str() {
        "gen-tree" => {
            let rest: Vec<String> = args.collect();
            let parsed = gen_tree::Args::parse(&rest)?;

            gen_tree::run(parsed)
        }
        "lint-style" => lint_style::run(),
        other => Err(XtaskError::UnknownCommand(other.to_owned())),
    }
}
