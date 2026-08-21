// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: GPL-3.0-only

// ── Imports ─────────────────────────────────────────────────────────────────
use anyhow::Result;

use clap::{Args, Subcommand};

use crate::types::CommandContext;

pub mod diff;
pub mod history;
pub mod list;
pub mod new;
pub mod prefixes;

// ── Args ─────────────────────────────────────────────────────────────────────
#[derive(Args)]
pub struct CommitArgs {
    #[command(subcommand)]
    pub command: CommitCommand,
}

// ── Subcommands ───────────────────────────────────────────────────────────────
#[derive(Subcommand)]
pub enum CommitCommand {
    Diff(diff::Args),
    History(history::Args),
    List(list::Args),
    New(new::Args),
    Prefixes(prefixes::Args),
}

// ── Dispatch ──────────────────────────────────────────────────────────────────
pub fn run(args: CommitArgs, context: CommandContext) -> Result<()> {
    match args.command {
        CommitCommand::Diff(args) => diff::run(args, context),
        CommitCommand::History(args) => history::run(args, context),
        CommitCommand::List(args) => list::run(args, context),
        CommitCommand::New(args) => new::run(args, context),
        CommitCommand::Prefixes(args) => prefixes::run(args, context),
    }
}
