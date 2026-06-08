// ── Imports ─────────────────────────────────────────────────────────────────
use anyhow::Result;

use clap::{Args, Subcommand};

use crate::types::CommandContext;

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
pub fn run(args: FileArgs, context: CommandContext) -> Result<()> {
    match args.command {
        FileCommand::Add(args) => add::run(args, context),
        FileCommand::Remove(args) => remove::run(args, context),
        FileCommand::Diff(args) => diff::run(args, context),
        FileCommand::Search(args) => search::run(args, context),
    }
}
