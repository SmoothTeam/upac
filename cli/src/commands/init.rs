use anyhow::Result;

use std::sync::Arc;

use crate::config::Config;
use crate::corelib::Lib;

#[derive(clap::Args)]
pub struct InitArgs {
    #[arg(long, default_value = "/etc/upac/config.toml")]
    pub config_path: String,
}

pub fn run(_args: InitArgs, _config: Config, _lib: Arc<Lib>) -> Result<()> {
    todo!()
}
