use anyhow::Result;

use crate::types::CommandContext;

#[derive(clap::Args)]
pub struct Args {
    #[arg(long)]
    pub full: bool,
}

pub fn run(_args: Args, _context: CommandContext) -> Result<()> {
    todo!()
}
