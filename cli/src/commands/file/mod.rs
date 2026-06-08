// ── Imports ─────────────────────────────────────────────────────────────────
use anyhow::Result;

use clap::{Args, Subcommand};

use std::sync::Arc;

use crate::config::Config;
use crate::corelib::Lib;

pub mod add;
pub mod diff;
pub mod remove;
pub mod search;

// ── Args ─────────────────────────────────────────────────────────────────────
#[derive(Args)]
pub struct FileArgs {
    #[command(subcommand)]
    pub command: FileCommand,
}

// ── Subcommands ───────────────────────────────────────────────────────────────
#[derive(Subcommand)]
pub enum FileCommand {
    Add(add::Args),
    Remove(remove::Args),
    Diff(diff::Args),
    Search(search::Args),
}

// ── Dispatch ──────────────────────────────────────────────────────────────────
pub fn run(args: FileArgs, config: Config, upac_lib: Arc<Lib>) -> Result<()> {
    match args.command {
        FileCommand::Add(args) => add::run(args, config, upac_lib),
        FileCommand::Remove(args) => remove::run(args, config, upac_lib),
        FileCommand::Diff(args) => diff::run(args, config, upac_lib),
        FileCommand::Search(args) => search::run(args, config, upac_lib),
    }
}
