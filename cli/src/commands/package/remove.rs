use anyhow::Result;

use std::ffi::CString;
use std::io::{self, Write};
use std::ptr::null_mut;

use colored::Colorize;

use crate::cancel_token_ptr;
use crate::ffi::packages::CPackageInfo;
use crate::ffi::request::{CMutatedRequest, CUnmutatedRequest, CUnmutatedResponse};
use crate::types::errors::LibError;
use crate::types::CommandContext;

type InstalledEntry = (String, String, Option<String>);
type ResolvedEntry = (CString, CString, Option<CString>);

#[derive(clap::Args)]
pub struct Args {
    #[arg(required = true, num_args = 1..)]
    pub names: Vec<String>,
    #[arg(long)]
    pub arch: Option<String>,
    #[arg(long)]
    pub arch_sub: Option<String>,
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
        let mut response = CUnmutatedResponse::empty();

        let request =
            CUnmutatedRequest::for_list_metas(&self.ctx.config.paths.root_path, cancel_token_ptr());

        let return_code = unsafe { (self.ctx.lib.pkg.list)(request, &mut response) };
        LibError::check(return_code)?;

        self.installed = unsafe { response.metas.as_slice() }
            .iter()
            .map(|package_meta| {
                (
                    unsafe { package_meta.name.as_str() }.to_owned(),
                    unsafe { package_meta.arch.as_str() }.to_owned(),
                    (!package_meta.arch_sub.ptr.is_null() && package_meta.arch_sub.len > 0)
                        .then(|| unsafe { package_meta.arch_sub.as_str() }.to_owned()),
                )
            })
            .collect();

        unsafe { (self.ctx.lib.free_response)(&mut response) };

        Ok(State::ResolvingFromInstalled)
    }

    fn state_resolving_direct(&mut self) -> Result<State> {
        let arch = self.args.arch.as_deref().unwrap();
        self.resolved = self
            .args
            .names
            .iter()
            .map(|name| {
                Ok((
                    CString::new(name.as_str())?,
                    CString::new(arch)?,
                    self.args
                        .arch_sub
                        .as_deref()
                        .map(CString::new)
                        .transpose()?,
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
        let packages_info: Vec<CPackageInfo> = self
            .resolved
            .iter()
            .map(|(name, arch, arch_sub)| CPackageInfo::new(name, arch, arch_sub.as_ref()))
            .collect();

        let request = CMutatedRequest::for_uninstall(
            &packages_info,
            &self.ctx.config.paths.repo_path,
            &self.ctx.config.paths.root_path,
            &self.ctx.config.ostree.branch,
            None,
            null_mut(),
            cancel_token_ptr(),
        );

        let return_code = unsafe { (self.ctx.lib.pkg.uninstall)(request) };
        LibError::check(return_code)?;

        Ok(State::Done)
    }
}

fn prompt_choice(name: &str, matches: &[&InstalledEntry]) -> Result<usize> {
    println!(
        "{} \"{}\":",
        gettextrs::gettext("multiple_found"),
        name.bold()
    );

    for (index, (_, arch, arch_sub)) in matches.iter().enumerate() {
        match arch_sub.as_deref() {
            Some(sub) => println!("  {}) {name} ({arch}/{sub})", index + 1),
            None => println!("  {}) {name} ({arch})", index + 1),
        }
    }

    print!("{} [1-{}]: ", gettextrs::gettext("choose"), matches.len());
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    let choice: usize = input
        .trim()
        .parse()
        .map_err(|_| anyhow::anyhow!(gettextrs::gettext("err_invalid_choice")))?;

    if choice < 1 || choice > matches.len() {
        anyhow::bail!(gettextrs::gettext("err_invalid_choice"));
    }

    Ok(choice - 1)
}

fn find_installed(installed: &[InstalledEntry], name: &str) -> Result<(String, Option<String>)> {
    let matches: Vec<&InstalledEntry> = installed.iter().filter(|(n, _, _)| n == name).collect();

    let entry = match matches.len() {
        0 => anyhow::bail!("{}: {name}", gettextrs::gettext("err_pkg_not_found")),
        1 => matches[0],
        _ => matches[prompt_choice(name, &matches)?],
    };

    Ok((entry.1.clone(), entry.2.clone()))
}
