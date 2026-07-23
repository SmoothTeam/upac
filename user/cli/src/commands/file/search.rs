use anyhow::Result;

use crate::types::CommandContext;

#[derive(clap::Args)]
pub struct Args {
    pub query: String,
}

pub fn run(_args: Args, _context: CommandContext) -> Result<()> {
    todo!()
}
