// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: GPL-3.0-only

use std::io::{self, Write};

use anyhow::Result;

use clap::Args as ClapArgs;

use colored::Colorize;

use i18n_embed_fl::fl;

use upac_abi::error::ErrorDomain;

use upac_types::package::{PackageInfo, PackageMeta};
use upac_types::request::mutated::UninstallRequest;
use upac_types::request::unmutated::ListPackagesRequest;

use crate::locale::{LOADER, SUBJECT_LOADER};
use crate::types::progress::ProgressState;
use crate::types::{CommandContext, boot_plugin, call, query, request_base};

#[cfg(test)]
#[path = "../../../tests/inline/remove.rs"]
mod tests;

#[derive(ClapArgs)]
pub struct Args {
    #[arg(required = true, num_args = 1..)]
    pub names: Vec<String>,
    #[arg(long)]
    pub arch: Option<String>,
    #[arg(long)]
    pub arch_sub: Option<String>,
    #[arg(short, long)]
    pub message: Option<String>,
    #[arg(long)]
    pub boot: Option<String>,
    #[arg(long)]
    pub purge: bool,
}

pub fn run(args: Args, ctx: CommandContext) -> Result<()> {
    let packages = match args.arch.as_deref() {
        Some(arch) => resolve_direct(&args, arch),
        None => resolve_from_installed(&args.names, &ctx)?,
    };

    uninstall(&args, &ctx, packages)
}

fn resolve_direct(args: &Args, arch: &str) -> Vec<PackageInfo> {
    args.names
        .iter()
        .map(|name| PackageInfo {
            name: name.clone(),
            arch: arch.to_owned(),
            arch_sub: args.arch_sub.clone(),
        })
        .collect()
}

fn resolve_from_installed(names: &[String], ctx: &CommandContext) -> Result<Vec<PackageInfo>> {
    let request = ListPackagesRequest { base: request_base!() };

    let installed: Vec<PackageMeta> = query!(ctx.lib.ro.list_packages, request, strict metas)?;

    names.iter().map(|name| select_installed(name, &installed)).collect()
}

fn select_installed(name: &str, installed: &[PackageMeta]) -> Result<PackageInfo> {
    let matches: Vec<&PackageMeta> = installed.iter().filter(|meta| meta.name == name).collect();

    let meta = match matches.len() {
        0 => anyhow::bail!("{}: {name}", fl!(LOADER, "err-pkg-not-found")),
        1 => matches[0],
        _ => matches[prompt_choice(name, &matches)?],
    };

    Ok(meta.clone().into())
}

fn uninstall(args: &Args, ctx: &CommandContext, packages: Vec<PackageInfo>) -> Result<()> {
    let symbols = ctx.lib.require_write()?;

    let mut progress = ProgressState::new(ErrorDomain::Uninstall);

    let boot_plugin = boot_plugin!(args.boot.clone())?;

    let subject = fl!(SUBJECT_LOADER, "subject-remove");

    let request = UninstallRequest {
        base: request_base!(progress),
        tmp_path: &ctx.tmp_path,
        subject: &subject,
        message: args.message.as_deref(),
        packages,
        boot_plugin: &boot_plugin,
        purge: args.purge,
    };

    call!(symbols.uninstall, request)
}

fn prompt_choice(name: &str, matches: &[&PackageMeta]) -> Result<usize> {
    println!("{} \"{}\":", fl!(LOADER, "multiple-found"), name.bold());

    for (index, meta) in matches.iter().enumerate() {
        match meta.arch_sub.as_deref() {
            Some(arch_sub) => println!("  {}) {name} ({}/{arch_sub})", index + 1, meta.arch),
            None => println!("  {}) {name} ({})", index + 1, meta.arch),
        }
    }

    print!("{} [1-{}]: ", fl!(LOADER, "choose"), matches.len());
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    let choice: usize = input
        .trim()
        .parse()
        .map_err(|_| anyhow::anyhow!(fl!(LOADER, "err-invalid-choice")))?;

    if choice < 1 || choice > matches.len() {
        anyhow::bail!(fl!(LOADER, "err-invalid-choice"));
    }

    Ok(choice - 1)
}
