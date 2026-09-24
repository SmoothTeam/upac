// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: GPL-3.0-only

use std::process::ExitCode;
use std::ptr::addr_of_mut;
use std::sync::Arc;

use anyhow::Result;

use clap::{Parser, Subcommand};

use colored::Colorize;

use i18n_embed_fl::fl;

use locale::{LOADER, init};

use upac_abi::hook::CancelToken;

use self::libcore::Lib;

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

static mut CANCEL_TOKEN: CancelToken = CancelToken::new();

pub(crate) fn cancel_token_ptr() -> *mut CancelToken {
    addr_of_mut!(CANCEL_TOKEN)
}

#[derive(Parser)]
#[command(name = "up-sp", author, version, about)]
struct Cli {
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
    let lib = Arc::new(Lib::load()?);

    let lib_cancel = Arc::clone(&lib);
    ctrlc::set_handler(move || {
        unsafe { (lib_cancel.cancel)(cancel_token_ptr()) };
    })?;

    let cli = Cli::parse();

    match cli.command {
        Command::Partition(args) => commands::partition::run(args, &lib)?,
        Command::Format(args) => commands::format::run(args, &lib)?,
        Command::Bootstrap(args) => commands::bootstrap::run(args, &lib)?,
    }

    Ok(())
}
