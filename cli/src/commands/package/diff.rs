use anyhow::Result;
use std::sync::Arc;

use crate::config::Config;
use crate::corelib::Lib;

#[derive(clap::Args)]
pub struct Args {
    pub from: Option<String>,
    pub to: Option<String>,
}

pub fn run(_args: Args, _config: Config, _upac_lib: Arc<Lib>) -> Result<()> {
    todo!()
}
