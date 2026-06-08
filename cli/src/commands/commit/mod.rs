// ── Imports ─────────────────────────────────────────────────────────────────
use anyhow::Result;

use clap::{Args, Subcommand};

use std::sync::Arc;

use crate::config::Config;
use crate::corelib::Lib;

pub mod list;
pub mod new;
pub mod rollback;

// ── Args ─────────────────────────────────────────────────────────────────────
#[derive(Args)]
pub struct CommitArgs {
    #[command(subcommand)]
    pub command: CommitCommand,
}

// ── Subcommands ───────────────────────────────────────────────────────────────
#[derive(Subcommand)]
pub enum CommitCommand {
    List(list::Args),
    New(new::Args),
    Rollback(rollback::Args),
}

// ── Dispatch ──────────────────────────────────────────────────────────────────
pub fn run(args: CommitArgs, config: Config, upac_lib: Arc<Lib>) -> Result<()> {
    match args.command {
        CommitCommand::List(args) => list::run(args, config, upac_lib),
        CommitCommand::New(args) => new::run(args, config, upac_lib),
        CommitCommand::Rollback(args) => rollback::run(args, config, upac_lib),
    }
}
