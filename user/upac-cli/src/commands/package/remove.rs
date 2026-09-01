// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: GPL-3.0-only

use std::ffi::CString;
use std::io::{self, Write};

use anyhow::Result;

use clap::Args as ClapArgs;

use colored::Colorize;

use i18n_embed_fl::fl;

use upac_abi::error::ErrorDomain;
use upac_abi::request::{CListPackagesRequest, CRequestBase, CUninstallRequest};
use upac_abi::types::CSlice;

use crate::cancel_token_ptr;
use crate::locale::LOADER;
use crate::types::CommandContext;
use crate::types::abi::{
    borrowed_vec, invoke, invoke_with_response, optional_slice, package_info, request_base, slice_from_cstr,
};
use crate::types::progress::{ProgressState, on_progress};

type InstalledEntry = (String, String, Option<String>);
type ResolvedEntry = (CString, CString, Option<CString>);

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
    let mut state = match args.arch.as_deref() {
        Some(_) => State::ResolvingDirect,
        None => State::Listing,
    };

    let mut machine = RemoveMachine {
        args,
        ctx,
        installed: Vec::new(),
        resolved: Vec::new(),
    };

    while state != State::Done {
        state = match state {
            State::Listing => machine.state_listing()?,
            State::ResolvingDirect => machine.state_resolving_direct()?,
            State::ResolvingFromInstalled => machine.state_resolving_from_installed()?,
            State::Removing => machine.state_removing()?,
            State::Done => unreachable!(),
        };
    }

    Ok(())
}

#[derive(PartialEq)]
enum State {
    Listing,
    ResolvingDirect,
    ResolvingFromInstalled,
    Removing,
    Done,
}

struct RemoveMachine {
    args: Args,
    ctx: CommandContext,

    installed: Vec<InstalledEntry>,
    resolved: Vec<ResolvedEntry>,
}

impl RemoveMachine {
    fn state_listing(&mut self) -> Result<State> {
        let request = CListPackagesRequest::new(request_base());

        let response =
            invoke_with_response(|out, error| unsafe { (self.ctx.lib.ro.list_packages)(request, out, error) })?;

        self.installed = unsafe { response.metas.as_slice() }
            .iter()
            .map(|package_meta| {
                Ok((
                    cslice_owned(&package_meta.name)?,
                    cslice_owned(&package_meta.arch)?,
                    optional_cslice_owned(&package_meta.arch_sub)?,
                ))
            })
            .collect::<Result<_>>()?;

        unsafe { response.free() };

        Ok(State::ResolvingFromInstalled)
    }

    fn state_resolving_direct(&mut self) -> Result<State> {
        let Some(arch) = self.args.arch.as_deref() else {
            anyhow::bail!(fl!(LOADER, "err-invalid-entry"));
        };

        self.resolved = self
            .args
            .names
            .iter()
            .map(|name| {
                Ok((
                    CString::new(name.as_str())?,
                    CString::new(arch)?,
                    self.args.arch_sub.as_deref().map(CString::new).transpose()?,
                ))
            })
            .collect::<Result<_>>()?;
        Ok(State::Removing)
    }

    fn state_resolving_from_installed(&mut self) -> Result<State> {
        self.resolved = self
            .args
            .names
            .iter()
            .map(|name| {
                let (arch, arch_sub) = find_installed(&self.installed, name)?;
                Ok((
                    CString::new(name.as_str())?,
                    CString::new(arch)?,
                    arch_sub.as_deref().map(CString::new).transpose()?,
                ))
            })
            .collect::<Result<_>>()?;
        Ok(State::Removing)
    }

    fn state_removing(&mut self) -> Result<State> {
        let symbols = self.ctx.lib.require_write()?;

        let subject = CString::new("remove")?;
        let message = self.args.message.as_deref().map(CString::new).transpose()?;
        let boot_plugin = self.args.boot.as_deref().map(CString::new).transpose()?;

        let packages: Vec<_> = self
            .resolved
            .iter()
            .map(|(name, arch, arch_sub)| package_info(name, arch, arch_sub.as_ref()))
            .collect();

        let mut progress = ProgressState::new(ErrorDomain::Uninstall);
        let base = CRequestBase::new(Some(on_progress), progress.ctx_ptr(), cancel_token_ptr());

        let request = CUninstallRequest::new(
            base,
            slice_from_cstr(&self.ctx.tmp_path),
            slice_from_cstr(&subject),
            optional_slice(message.as_ref()),
            borrowed_vec(&packages),
            optional_slice(boot_plugin.as_ref()),
            self.args.purge,
        );

        let result = invoke(|error| unsafe { (symbols.uninstall)(request, error) });
        progress.finish();
        result?;

        Ok(State::Done)
    }
}

fn prompt_choice(name: &str, matches: &[&InstalledEntry]) -> Result<usize> {
    println!("{} \"{}\":", fl!(LOADER, "multiple-found"), name.bold());

    for (index, (_, arch, arch_sub)) in matches.iter().enumerate() {
        match arch_sub.as_deref() {
            Some(sub) => println!("  {}) {name} ({arch}/{sub})", index + 1),
            None => println!("  {}) {name} ({arch})", index + 1),
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

fn find_installed(installed: &[InstalledEntry], name: &str) -> Result<(String, Option<String>)> {
    let matches: Vec<&InstalledEntry> = installed.iter().filter(|(n, _, _)| n == name).collect();

    let entry = match matches.len() {
        0 => anyhow::bail!("{}: {name}", fl!(LOADER, "err-pkg-not-found")),
        1 => matches[0],
        _ => matches[prompt_choice(name, &matches)?],
    };

    Ok((entry.1.clone(), entry.2.clone()))
}

fn cslice_owned(slice: &CSlice) -> Result<String> {
    Ok(unsafe { slice.as_str() }
        .map_err(|_| anyhow::anyhow!(fl!(LOADER, "err-invalid-entry")))?
        .to_owned())
}

fn optional_cslice_owned(slice: &CSlice) -> Result<Option<String>> {
    if slice.ptr.is_null() || slice.len == 0 {
        return Ok(None);
    }
    Ok(Some(cslice_owned(slice)?))
}
