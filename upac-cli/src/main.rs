// ── Imports ─────────────────────────────────────────────────────────────────
use anyhow::Result;

use clap::{Parser, Subcommand};

use colored::Colorize;

use std::fs;
use std::path::{Path, PathBuf};
use std::ptr::addr_of_mut;
use std::sync::Arc;

use crate::ffi::CancelToken;
use crate::upac::UpacLib;
use crate::utils::BackendKind;

use commands::diff::DiffArgs;
use commands::init::InitArgs;
use commands::install::InstallArgs;
use commands::list::ListArgs;
use commands::remove::RemoveArgs;
use commands::rollback::RollbackArgs;

use config::Config;

mod backends;
mod config;
mod upac;

pub mod ffi;
pub mod utils;

mod commands {
    pub mod install;
    pub mod remove;
    pub mod rollback;

    pub mod diff;
    pub mod list;

    pub mod init;
}

static mut CANCEL_TOKEN: CancelToken = CancelToken::new();

pub(crate) fn cancel_token_ptr() -> *mut CancelToken {
    addr_of_mut!(CANCEL_TOKEN)
}

// ── CLI arguments ─────────────────────────────────────────────────────────────
// Automatic generation of Cli structure parser
#[derive(Parser)]
#[command(name = "upac", about = "A modular Linux package manager", version)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

// Enumerate all available CLI subcommands
#[derive(Subcommand)]
enum Command {
    Install(InstallArgs),
    Remove(RemoveArgs),
    Rollback(RollbackArgs),

    List(ListArgs),
    Diff(DiffArgs),
    Init(InitArgs),
}

// ── Entry points ───────────────────────────────────────────────────────────────
// The main entry point, responsible for error output and the return code.
// Spawned on a large stack because install/uninstall invoke OSTree C functions
// that recurse deeply through large package directory trees.
fn main() {
    let result = run();
    match result {
        Ok(()) => {}
        Err(err) => {
            eprintln!("{} {err}", "Error:".red().bold());
        }
    }
}

// Core business logic: argument parsing, config loading, and command execution
fn run() -> Result<()> {
    let default_config_path =
        check_default_config_path().ok_or(anyhow::anyhow!("no default config path found"))?;
    let config = Config::load(&default_config_path)?;

    let cli = Cli::parse();

    let upac_lib = Arc::new(UpacLib::load(&BackendKind::UpacLib)?);

    let lib_cancel = Arc::clone(&upac_lib);
    ctrlc::set_handler(move || {
        unsafe { (lib_cancel.cancel)(cancel_token_ptr()) };
    })?;

    match cli.command {
        Command::Install(args) => {
            commands::install::run(args, config, upac_lib)?;
        }
        Command::Remove(args) => {
            commands::remove::run(args, config, upac_lib)?;
        }
        Command::List(args) => {
            commands::list::run(args, config, upac_lib)?;
        }
        Command::Diff(args) => {
            commands::diff::run(args, config, upac_lib)?;
        }
        Command::Rollback(args) => {
            commands::rollback::run(args, config, upac_lib)?;
        }
        Command::Init(args) => {
            commands::init::run(args, config, upac_lib)?;
        }
    }

    Ok(())
}

// ── Helpers ───────────────────────────────────────────────────────────────────
// Standard path validation function
fn check_default_config_path() -> Option<PathBuf> {
    let path = Path::new("/etc/upac/config.toml");

    if fs::metadata(path).is_ok() {
        Some(path.to_path_buf())
    } else {
        None
    }
}
