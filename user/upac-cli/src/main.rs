// SPDX-FileCopyrightText: 2026 JustPav
//
// SPDX-License-Identifier: GPL-3.0-only

// ── Imports ─────────────────────────────────────────────────────────────────
use std::process::ExitCode;
use std::ptr::addr_of_mut;
use std::sync::Arc;

use gettextrs::{LocaleCategory, bindtextdomain, setlocale, textdomain};

use colored::Colorize;

use anyhow::Result;

use clap::Parser;

use upac_abi::hook::CancelToken;

use crate::commands::commit::CommitArgs;
use crate::commands::file::FileArgs;
use crate::commands::package::PkgArgs;
use crate::libcore::Lib;
use crate::types::CommandContext;

mod libcore;
mod types;

mod commands {
    pub mod commit;
    pub mod display;
    pub mod file;
    pub mod gc;
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
    Commit(CommitArgs),
    File(FileArgs),
    Gc(commands::gc::Args),
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
    let lib = Arc::new(Lib::load()?);

    let lib_cancel = Arc::clone(&lib);
    ctrlc::set_handler(move || {
        unsafe { (lib_cancel.cancel)(cancel_token_ptr()) };
    })?;

    let command_context = CommandContext::new(lib)?;

    match Command::parse() {
        Command::Pkg(args) => commands::package::run(args, command_context)?,
        Command::Commit(args) => commands::commit::run(args, command_context)?,
        Command::File(args) => commands::file::run(args, command_context)?,
        Command::Gc(args) => commands::gc::run(args, command_context)?,
    }

    Ok(())
}
