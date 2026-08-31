// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: GPL-3.0-only

// ── Imports ─────────────────────────────────────────────────────────────────
use std::env::{set_var, var};
use std::process::ExitCode;

use anyhow::Result;

use gettextrs::{LocaleCategory, bindtextdomain, setlocale, textdomain};

use clap::{Parser, Subcommand};

use colored::Colorize;

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
    let lang = var("LANG")
        .ok()
        .filter(|value| value.starts_with("ru"))
        .map_or("en", |_| "ru");

    // SAFETY: called first thing in main, before any other threads or signal handlers exist.
    unsafe {
        setlocale(LocaleCategory::LcAll, "C.utf8");
        set_var("LANGUAGE", lang);
    }

    let locale_dir = locale::extract().expect("failed to extract embedded locale data");
    bindtextdomain("upac-setup", &locale_dir).expect("bindtextdomain failed");
    textdomain("upac-setup").expect("textdomain failed");

    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("{} {err}", format!("{}:", gettextrs::gettext("error")).red().bold());
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
