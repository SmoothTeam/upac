// SPDX-FileCopyrightText: 2026 JustPav
//
// SPDX-License-Identifier: GPL-3.0-only

use anyhow::Result;

use crate::types::CommandContext;

#[derive(clap::Args)]
pub struct Args {
    pub from: Option<String>,
    pub to: Option<String>,
}

pub fn run(_args: Args, _context: CommandContext) -> Result<()> {
    todo!()
}
