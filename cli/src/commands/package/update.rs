use anyhow::Result;

use crate::types::CommandContext;

#[derive(clap::Args)]
pub struct Args {
    #[arg(required = true, num_args = 1..)]
    pub files: Vec<String>,
    #[arg(long)]
    pub backend: Option<String>,
    #[arg(long, num_args = 0..)]
    pub checksums: Vec<String>,
}

pub fn run(_args: Args, _context: CommandContext) -> Result<()> {
    todo!()
}
