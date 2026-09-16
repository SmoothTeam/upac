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

use upac_abi::hook::CancelToken;

use self::libcore::Lib;

mod commands {
    pub mod auto;
    pub mod manual;
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
    command: Option<Command>,

    #[command(flatten)]
    whole_disk: commands::auto::Args,
}

#[derive(Subcommand)]
enum Command {
    Manual(commands::manual::Args),
}

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
    let lib = Arc::new(Lib::load()?);

    let lib_cancel = Arc::clone(&lib);
    ctrlc::set_handler(move || {
        unsafe { (lib_cancel.cancel)(cancel_token_ptr()) };
    })?;

    let cli = Cli::parse();

    match cli.command {
        Some(Command::Manual(args)) => commands::manual::run(args, &lib)?,
        None => commands::auto::run(cli.whole_disk, &lib)?,
    }

    Ok(())
}
