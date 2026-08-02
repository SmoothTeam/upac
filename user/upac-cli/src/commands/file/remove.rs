use anyhow::Result;

use crate::types::CommandContext;

#[derive(clap::Args)]
pub struct Args {
    #[arg(required = true, num_args = 1..)]
    pub files: Vec<String>,
    #[arg(long, required = true)]
    pub package: String,
}

pub fn run(_args: Args, _context: CommandContext) -> Result<()> {
    todo!()
}
