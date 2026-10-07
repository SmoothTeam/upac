// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: GPL-3.0-only

use std::process::ExitCode;

use anyhow::Result;

use clap::{Parser, Subcommand};

use colored::Colorize;

use i18n_embed_fl::fl;

use locale::{LOADER, Locale, init};

use upac_types::CancelToken;

use upac_locale::parse;

use self::libcore::Lib;
use self::types::output::Output;

mod commands {
    pub mod bootstrap;
    pub mod format;
    pub mod partition;
}

mod libcore;
mod locale;
mod layout {
    include!(concat!(env!("OUT_DIR"), "/layout.rs"));
}
mod types;

static CANCEL_TOKEN: CancelToken = CancelToken::new();

#[derive(Parser)]
#[command(name = "up-sp", author, version)]
struct Cli {
    #[arg(long, global = true)]
    porcelain: bool,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    Partition(commands::partition::Args),
    Format(commands::format::Args),
    Bootstrap(commands::bootstrap::Args),
}

fn main() -> ExitCode {
    init();

    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("{} {err}", format!("{}:", fl!(LOADER, "error")).red().bold());
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<()> {
    let cli = parse::<Cli, Locale>();

    let lib = Lib::load()?;
    let output = Output::new(cli.porcelain);

    ctrlc::set_handler(|| CANCEL_TOKEN.cancel())?;

    match cli.command {
        Command::Partition(args) => commands::partition::run(args, &lib, &output)?,
        Command::Format(args) => commands::format::run(args, &lib)?,
        Command::Bootstrap(args) => commands::bootstrap::run(args, &lib, &output)?,
    }

    Ok(())
}
