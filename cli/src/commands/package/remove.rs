use anyhow::Result;

use crate::types::CommandContext;

#[derive(clap::Args)]
pub struct Args {
    #[arg(required = true, num_args = 1..)]
    pub names: Vec<String>,
}

pub fn run(_args: Args, _context: CommandContext) -> Result<()> {
    todo!()
}
