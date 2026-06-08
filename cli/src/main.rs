// ── Imports ─────────────────────────────────────────────────────────────────
use anyhow::Result;

use gettextrs::{bindtextdomain, setlocale, textdomain, LocaleCategory};

use clap::Parser;

use colored::Colorize;

use std::path::Path;
use std::ptr::addr_of_mut;

use std::sync::Arc;

use crate::corelib::Lib;
use crate::ffi::CancelToken;

use commands::commit::CommitArgs;
use commands::file::FileArgs;
use commands::init::InitArgs;
use commands::package::PkgArgs;

use config::Config;

mod config;
pub mod corelib;
pub mod ffi;
pub mod types;

mod commands {
    pub mod commit;
    pub mod file;
    pub mod init;
    pub mod package;
    pub mod utils;
}

static mut CANCEL_TOKEN: CancelToken = CancelToken::new();

pub(crate) fn cancel_token_ptr() -> *mut CancelToken {
    addr_of_mut!(CANCEL_TOKEN)
}

// ── CLI arguments ─────────────────────────────────────────────────────────────
#[derive(Parser)]
#[command(author, version, about)]
enum Command {
    Commit(CommitArgs),
    Pkg(PkgArgs),
    File(FileArgs),
    Init(InitArgs),
}

// ── Entry points ───────────────────────────────────────────────────────────────
fn main() {
    setlocale(LocaleCategory::LcAll, "");

    bindtextdomain("upac", env!("LOCALEDIR")).expect("bindtextdomain failed");
    textdomain("upac").expect("textdomain failed");

    let result = run();
    match result {
        Ok(()) => {}
        Err(err) => {
            eprintln!(
                "{} {err}",
                format!("{}:", gettextrs::gettext("error")).red().bold()
            );
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

    match Command::parse() {
        Command::Commit(args) => commands::commit::run(args, config, lib)?,
        Command::Pkg(args) => commands::package::run(args, config, lib)?,
        Command::File(args) => commands::file::run(args, config, lib)?,
        Command::Init(args) => commands::init::run(args, config, lib)?,
    }

    Ok(())
}
