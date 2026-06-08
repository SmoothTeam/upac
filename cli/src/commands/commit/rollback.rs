use anyhow::Result;

use crate::types::CommandContext;

#[derive(clap::Args)]
pub struct Args {
    pub commit: String,
}

pub fn run(_args: Args, _context: CommandContext) -> Result<()> {
    todo!()
}
