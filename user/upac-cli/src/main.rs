// SPDX-FileCopyrightText: 2026 JustPav
//
// SPDX-License-Identifier: GPL-3.0-only

// ── Imports ─────────────────────────────────────────────────────────────────
use std::path::Path;
use std::process::ExitCode;
use std::ptr::addr_of_mut;
use std::sync::Arc;

use gettextrs::{LocaleCategory, bindtextdomain, setlocale, textdomain};

use colored::Colorize;

use anyhow::Result;

use clap::Parser;

use upac_abi::hook::CancelToken;

use crate::commands::package::PkgArgs;
use crate::config::Config;
use crate::libcore::Lib;
use crate::types::CommandContext;

mod config;
mod libcore;
mod types;

mod commands {
    pub mod display;
    pub mod package;
}

static mut CANCEL_TOKEN: CancelToken = CancelToken::new();

pub(crate) fn cancel_token_ptr() -> *mut CancelToken {
    addr_of_mut!(CANCEL_TOKEN)
}

// ── CLI arguments ─────────────────────────────────────────────────────────────
#[derive(Parser)]
#[command(author, version, about)]
enum Command {
    Pkg(PkgArgs),
}

// ── Entry points ───────────────────────────────────────────────────────────────
fn main() -> ExitCode {
    setlocale(LocaleCategory::LcAll, "");

    bindtextdomain("upac", env!("LOCALEDIR")).expect("bindtextdomain failed");
    textdomain("upac").expect("textdomain failed");

    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("{} {err}", format!("{}:", gettextrs::gettext("error")).red().bold());
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<()> {
    let config = Config::load(Path::new(config::DEFAULT_CONFIG_PATH))?;

    let lib = Arc::new(Lib::load()?);

    let lib_cancel = Arc::clone(&lib);
    ctrlc::set_handler(move || {
        unsafe { (lib_cancel.cancel)(cancel_token_ptr()) };
    })?;

    let command_context = CommandContext::new(config, lib)?;

    match Command::parse() {
        Command::Pkg(args) => commands::package::run(args, command_context)?,
    }

    Ok(())
}
