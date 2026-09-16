// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: GPL-3.0-only

use std::process::ExitCode;

use clap::Parser;

use crate::error::XtaskError;

mod error;
mod hook;
mod lint;
mod tree;

#[derive(Parser)]
#[command(author, version, about)]
enum Command {
    /// Regenerate the ASCII directory tree embedded in docs
    Tree(tree::Args),
    /// Check mechanical style-rule violations across the repo
    Lint,
    /// Build composefs-setup-root (crates.io, no [lib] target) into target/
    Hook,
}

fn main() -> ExitCode {
    let command = Command::parse();

    match dispatch(command) {
        Ok(code) => code,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

fn dispatch(command: Command) -> Result<ExitCode, XtaskError> {
    match command {
        Command::Tree(args) => tree::run(args),
        Command::Lint => lint::run(),
        Command::Hook => hook::run(),
    }
}
