// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: GPL-3.0-only

use std::process::ExitCode;

use clap::{Parser, Subcommand};

use upac_xtask::error::XtaskError;
use upac_xtask::{gen_tree, lint_style};

#[derive(Parser)]
#[command(name = "xtask")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Regenerate the ASCII directory tree embedded in docs
    GenTree(gen_tree::Args),
    /// Check mechanical style-rule violations across the repo
    LintStyle,
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    match dispatch(cli.command) {
        Ok(code) => code,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

fn dispatch(command: Command) -> Result<ExitCode, XtaskError> {
    match command {
        Command::GenTree(args) => gen_tree::run(args),
        Command::LintStyle => lint_style::run(),
    }
}
