use anyhow::Result;
use std::sync::Arc;

use crate::types::CommandContext;

use crate::config::Config;
use crate::corelib::Lib;

#[derive(clap::Args)]
pub struct Args {
    pub message: String,
}

pub fn run(_args: Args, _context: CommandContext) -> Result<()> {
    todo!()
}
