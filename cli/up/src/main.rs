// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: GPL-3.0-only

use std::process::ExitCode;

use colored::Colorize;

use anyhow::Result;

use clap::Parser;

use i18n_embed_fl::fl;

use upac_types::CancelToken;

use upac_locale::parse;

use self::commands::commit::CommitArgs;
use self::commands::file::FileArgs;
use self::commands::package::PkgArgs;
use self::libcore::Lib;
use self::locale::Locale;
use self::types::CommandContext;

mod libcore;
mod layout {
    include!(concat!(env!("OUT_DIR"), "/layout.rs"));
}
mod locale;
mod types;

mod commands {
    pub mod commit;
    pub mod diff;
    pub mod display;
    pub mod file;
    pub mod gc;
    pub mod mime;
    pub mod package;
    pub mod rollback;
}

static CANCEL_TOKEN: CancelToken = CancelToken::new();

#[derive(Parser)]
#[command(author, version)]
enum Command {
    Pkg(PkgArgs),
    Commit(CommitArgs),
    File(FileArgs),
    Gc(commands::gc::Args),
    Diff(commands::diff::Args),
    Mime(commands::mime::MimeArgs),
    Rollback(commands::rollback::Args),
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
    let command = parse::<Command, Locale>();

    let lib = Lib::load()?;

    ctrlc::set_handler(|| CANCEL_TOKEN.cancel())?;

    let command_context = CommandContext::new(lib);

    match command {
        Command::Pkg(args) => commands::package::run(args, command_context)?,
        Command::Commit(args) => commands::commit::run(args, command_context)?,
        Command::File(args) => commands::file::run(args, command_context)?,
        Command::Gc(args) => commands::gc::run(args, command_context)?,
        Command::Diff(args) => commands::diff::run(args, command_context)?,
        Command::Mime(args) => commands::mime::run(args, command_context)?,
        Command::Rollback(args) => commands::rollback::run(args, command_context)?,
    }

    Ok(())
}
