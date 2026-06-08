// ── Imports ─────────────────────────────────────────────────────────────────
use anyhow::Result;

use clap::{Args, Subcommand};

use crate::types::CommandContext;

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
pub fn run(args: CommitArgs, context: CommandContext) -> Result<()> {
    match args.command {
        CommitCommand::List(args) => list::run(args, context),
        CommitCommand::New(args) => new::run(args, context),
        CommitCommand::Rollback(args) => rollback::run(args, context),
    }
}
