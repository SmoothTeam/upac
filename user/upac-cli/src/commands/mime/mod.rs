// SPDX-FileCopyrightText: 2026 JustPav
//
// SPDX-License-Identifier: GPL-3.0-only

// ── Imports ─────────────────────────────────────────────────────────────────
use anyhow::Result;

use clap::{Args, Subcommand};

use crate::types::CommandContext;

pub mod sync;

// ── Args ─────────────────────────────────────────────────────────────────────
#[derive(Args)]
pub struct MimeArgs {
    #[command(subcommand)]
    pub command: MimeCommand,
}

// ── Subcommands ───────────────────────────────────────────────────────────────
#[derive(Subcommand)]
pub enum MimeCommand {
    Sync(sync::Args),
}

// ── Dispatch ──────────────────────────────────────────────────────────────────
pub fn run(args: MimeArgs, context: CommandContext) -> Result<()> {
    match args.command {
        MimeCommand::Sync(args) => sync::run(args, context),
    }
}
