// ── Imports ─────────────────────────────────────────────────────────────────
use anyhow::Result;

use clap::{Parser, Subcommand};

use colored::Colorize;

use std::fs;
use std::path::{Path, PathBuf};
use std::ptr::null_mut;
use std::sync::atomic::{AtomicPtr, Ordering};

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

pub static CURRENT_CANCEL_TOKEN: AtomicPtr<CancelToken> = AtomicPtr::new(null_mut());

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
    let cli = Cli::parse();
    ctrlc::set_handler(|| {
        let token = CURRENT_CANCEL_TOKEN.load(Ordering::SeqCst);
        if !token.is_null() {
            ffi::cancel_token(token);
        }
    })?;

    let default_config_path =
        check_default_config_path().ok_or(anyhow::anyhow!("no default config path found"))?;
    let config = Config::load(&default_config_path)?;

    match cli.command {
        Command::Install(args) => {
            commands::install::run(config, args)?;
        }
        Command::Remove(args) => {
            commands::remove::run(config, args)?;
        }
        Command::List(args) => {
            commands::list::run(config, args)?;
        }
        Command::Diff(args) => {
            commands::diff::run(config, args)?;
        }
        Command::Rollback(args) => {
            commands::rollback::run(config, args)?;
        }
        Command::Init(args) => {
            commands::init::run(config, args)?;
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
