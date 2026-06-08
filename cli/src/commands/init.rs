use anyhow::Result;

use crate::types::CommandContext;

#[derive(clap::Args)]
pub struct InitArgs {
    #[arg(long, default_value = "/etc/upac/config.toml")]
    pub config_path: String,
}

pub fn run(_args: InitArgs, _context: CommandContext) -> Result<()> {
    todo!()
}
