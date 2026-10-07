// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: GPL-3.0-only

use anyhow::Result;

use clap::{Args as ClapArgs, Subcommand};

use crate::libcore::Lib;

pub mod create;
pub mod esp;

#[derive(ClapArgs)]
pub struct Args {
    #[command(subcommand)]
    pub command: FormatCommand,
}

#[derive(Subcommand)]
pub enum FormatCommand {
    Esp(esp::Args),
    Create(create::Args),
}

pub fn run(args: Args, lib: &Lib) -> Result<()> {
    match args.command {
        FormatCommand::Esp(args) => esp::run(args, lib),
        FormatCommand::Create(args) => create::run(args, lib),
    }
}
