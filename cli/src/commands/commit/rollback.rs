use anyhow::Result;
use std::sync::Arc;

use crate::config::Config;
use crate::corelib::Lib;

#[derive(clap::Args)]
pub struct Args {
    pub commit: String,
}

pub fn run(_args: Args, _config: Config, _upac_lib: Arc<Lib>) -> Result<()> {
    todo!()
}
