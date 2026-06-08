// ── Imports ─────────────────────────────────────────────────────────────────
use anyhow::Result;

use clap::{Args, Subcommand};

use std::sync::Arc;

use crate::config::Config;
use crate::corelib::Lib;

pub mod diff;
pub mod install;
pub mod list;
pub mod remove;
pub mod search;
pub mod update;

// ── Args ─────────────────────────────────────────────────────────────────────
#[derive(Args)]
pub struct PkgArgs {
    #[command(subcommand)]
    pub command: PkgCommand,
}

// ── Subcommands ───────────────────────────────────────────────────────────────
#[derive(Subcommand)]
pub enum PkgCommand {
    Install(install::Args),
    #[command(alias = "uninstall")]
    Remove(remove::Args),
    Update(update::Args),
    List(list::Args),
    Diff(diff::Args),
    Search(search::Args),
}

// ── Dispatch ──────────────────────────────────────────────────────────────────
pub fn run(args: PkgArgs, config: Config, upac_lib: Arc<Lib>) -> Result<()> {
    match args.command {
        PkgCommand::Install(args) => install::run(args, config, upac_lib),
        PkgCommand::Remove(args) => remove::run(args, config, upac_lib),
        PkgCommand::Update(args) => update::run(args, config, upac_lib),
        PkgCommand::List(args) => list::run(args, config, upac_lib),
        PkgCommand::Diff(args) => diff::run(args, config, upac_lib),
        PkgCommand::Search(args) => search::run(args, config, upac_lib),
    }
}
