// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: GPL-3.0-only

use anyhow::Result;

use clap::{Args as ClapArgs, Subcommand};

use crate::libcore::Lib;
use crate::types::output::Output;

pub mod deploy;
pub mod import;
pub mod kernel;

#[derive(ClapArgs)]
pub struct Args {
    #[command(subcommand)]
    pub command: BootstrapCommand,
}

#[derive(Subcommand)]
pub enum BootstrapCommand {
    Import(import::Args),
    Kernel(kernel::Args),
    Deploy(deploy::Args),
}

pub fn run(args: Args, lib: &Lib, output: &Output) -> Result<()> {
    match args.command {
        BootstrapCommand::Import(args) => import::run(args, lib, output),
        BootstrapCommand::Kernel(args) => kernel::run(args, lib, output),
        BootstrapCommand::Deploy(args) => deploy::run(args, lib),
    }
}
