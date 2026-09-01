// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: GPL-3.0-only

// ── Imports ─────────────────────────────────────────────────────────────────
use std::process::ExitCode;

use anyhow::Result;

use clap::{Parser, Subcommand};

use colored::Colorize;

use i18n_embed_fl::fl;

use upac_abi::hook::CancelToken;

mod commands {
    pub mod manual;
    pub mod whole_disk;
}

mod errors;
mod locale;
mod progress;
mod types;

static CANCEL_TOKEN: CancelToken = CancelToken::new();

// ── CLI arguments ─────────────────────────────────────────────────────────────
#[derive(Parser)]
#[command(name = "up-sp", author, version, about)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,

    #[command(flatten)]
    whole_disk: commands::whole_disk::Args,
}

#[derive(Subcommand)]
enum Command {
    Manual(commands::manual::Args),
}

// ── Entry points ───────────────────────────────────────────────────────────────
fn main() -> ExitCode {
    locale::init();

    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("{} {err}", format!("{}:", fl!(locale::LOADER, "error")).red().bold());
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<()> {
    ctrlc::set_handler(|| CANCEL_TOKEN.cancel())?;

    let cli = Cli::parse();

    match cli.command {
        Some(Command::Manual(args)) => commands::manual::run(args, &CANCEL_TOKEN)?,
        None => commands::whole_disk::run(cli.whole_disk, &CANCEL_TOKEN)?,
    }

    Ok(())
}
