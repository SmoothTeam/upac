// SPDX-FileCopyrightText: 2026 JustPav
//
// SPDX-License-Identifier: GPL-3.0-only

// ── Imports ─────────────────────────────────────────────────────────────────
use anyhow::Result;

use gettextrs::{LocaleCategory, bindtextdomain, setlocale, textdomain};

use clap::Parser;

use colored::Colorize;

mod commands {
    pub mod generate_cert;
    pub mod generate_root;
    pub mod sign_hook;
    pub mod verify_hook;
}

mod errors;

// ── CLI arguments ─────────────────────────────────────────────────────────────
#[derive(Parser)]
#[command(name = "up-si", author, version, about)]
enum Command {
    GenerateRoot(commands::generate_root::Args),
    GenerateCert(commands::generate_cert::Args),
    SignHook(commands::sign_hook::Args),
    VerifyHook(commands::verify_hook::Args),
}

// ── Entry points ───────────────────────────────────────────────────────────────
fn main() {
    setlocale(LocaleCategory::LcAll, "");

    bindtextdomain("upac-sign", env!("LOCALEDIR")).expect("bindtextdomain failed");
    textdomain("upac-sign").expect("textdomain failed");

    let result = run();
    match result {
        Ok(()) => {}
        Err(err) => {
            eprintln!("{} {err}", format!("{}:", gettextrs::gettext("error")).red().bold());
        }
    }
}

fn run() -> Result<()> {
    match Command::parse() {
        Command::GenerateRoot(args) => commands::generate_root::run(args)?,
        Command::GenerateCert(args) => commands::generate_cert::run(args)?,
        Command::SignHook(args) => commands::sign_hook::run(args)?,
        Command::VerifyHook(args) => commands::verify_hook::run(args)?,
    }

    Ok(())
}
