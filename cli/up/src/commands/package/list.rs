// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: GPL-3.0-only

use anyhow::Result;

use clap::Args as ClapArgs;

use upac_types::package::PackageMeta;
use upac_types::request::unmutated::ListPackagesRequest;

use crate::commands::display::DisplayPakcageMetaArgs;
use crate::types::{CommandContext, query, request_base};

#[derive(ClapArgs)]
pub struct Args {
    #[command(flatten)]
    pub display_package_meta: DisplayPakcageMetaArgs,
}

pub fn run(args: Args, ctx: CommandContext) -> Result<()> {
    let request = ListPackagesRequest { base: request_base!() };

    let metas: Vec<PackageMeta> = query!(ctx.lib.ro.list_packages, request, metas)?;

    args.display_package_meta.print(&metas);

    Ok(())
}
