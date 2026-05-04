// ── Imports ─────────────────────────────────────────────────────────────────
use anyhow::Result;

use colored::Colorize;

use std::ptr::null_mut;
use std::sync::Arc;

use crate::cancel_token_ptr;
use crate::config::Config;
use crate::ffi::{CArray, CSlice, CUnmutatedRequest, CommitHandle, PackageMetaHandle};
use crate::upac::UpacLib;
use crate::utils::PackageField;

macro_rules! get_package_handle {
    ($lib:expr, $array:expr, $idx:expr, $err:expr) => {{
        let mut handle_ptr = null_mut();
        UpacLib::check(($lib.get_package_at)($array, $idx, &mut handle_ptr), $err)?;
        handle_ptr
    }};
}

macro_rules! get_package_field {
    ($lib:expr, $handle:expr, $field:expr, @int $dest:expr) => {{
        UpacLib::check(
            ($lib.get_package_int_field)($handle, $field as u8, &mut $dest),
            &format!("get int field {:?}", $field),
        )
    }};

    ($lib:expr, $handle:expr, $field:expr) => {{
        let mut slice = CSlice::empty();
        UpacLib::check(
            ($lib.get_package_slice_field)($handle, $field as u8, &mut slice),
            &format!("get str field {}", $field.as_str()),
        )?;
        slice.as_str().to_owned()
    }};
}

macro_rules! get_commit_handle {
    ($lib:expr, $array:expr, $idx:expr, $err:expr) => {{
        let mut handle_ptr = null_mut();
        UpacLib::check(($lib.get_commit_at)($array, $idx, &mut handle_ptr), $err)?;
        handle_ptr
    }};
}

macro_rules! get_commit_field {
    ($lib:expr, $handle:expr, $idx:expr, $err:expr) => {{
        let mut slice = CSlice::empty();
        UpacLib::check(
            ($lib.get_commit_slice_field)($handle, $idx, &mut slice),
            $err,
        )?;
        slice.as_str().to_owned()
    }};
}

// ── Row types ────────────────────────────────────────────────────────────────────────
struct PackageRow {
    name: String,
    version: String,
    size: u32,
    architecture: String,
    author: String,
    license: String,
    url: String,
    packager: String,
}

struct CommitRow {
    checksum: String,
    subject: String,
}

impl CommitRow {
    pub fn new(checksum: String, subject: String) -> Self {
        Self { checksum, subject }
    }
}

// ── Arguments for command ───────────────────────────────────────────────────────────────────────
#[derive(clap::Args)]
pub struct ListArgs {
    #[arg(long)]
    pub commit: bool,
    #[arg(long)]
    pub full: bool,
}

// ── FSM states ────────────────────────────────────────────────────────────────────────
#[derive(Debug, Clone, PartialEq)]
enum State {
    Starting,
    GetPackages,
    GetCommits,
    PrintCommits,
    PrintPackages,

    Done,
}

// ── FSM machine ────────────────────────────────────────────────────────────────────────
struct ListMachine {
    full: bool,
    commits_mode: bool,

    commits: Vec<CommitRow>,
    packages: Vec<PackageRow>,

    config: Config,
    upac_lib: Arc<UpacLib>,
    state: State,
}

impl ListMachine {
    fn new(config: Config, upac_lib: Arc<UpacLib>, commits_mode: bool, full: bool) -> Result<Self> {
        Ok(Self {
            full,

            commits_mode,
            packages: Vec::new(),

            commits: Vec::new(),

            config,
            upac_lib: upac_lib,
            state: State::Starting,
        })
    }
}

// ── Public API ─────────────────────────────────────────────────────────────
pub fn run(args: ListArgs, config: Config, upac_lib: Arc<UpacLib>) -> Result<()> {
    let mut list_machine = ListMachine::new(config, upac_lib, args.commit, args.full)?;

    match list_machine.commits_mode {
        true => state_get_commits_info(&mut list_machine).map_err(|err| {
            if list_machine.config.verbose {
                eprintln!(
                    "{} failed at state {:?}",
                    "✗".red().bold(),
                    list_machine.state
                );
            }
            err
        }),
        false => state_get_packages_list(&mut list_machine).map_err(|err| {
            if list_machine.config.verbose {
                eprintln!(
                    "{} failed at state {:?}",
                    "✗".red().bold(),
                    list_machine.state
                );
            }
            err
        }),
    }
}

// ── States ─────────────────────────────────────────────────────────────────
fn state_get_commits_info(machine: &mut ListMachine) -> Result<()> {
    machine.state = State::GetCommits;

    let mut commit_array_c: CArray<CommitHandle> = CArray::empty();

    let token_ptr = cancel_token_ptr();

    let list_request_c = CUnmutatedRequest::for_list(
        &machine.config.paths.repo_path,
        &machine.config.paths.root_path,
        &machine.config.paths.database_path,
        &machine.config.ostree.branch,
        &machine.config.ostree.prefix_directory,
        token_ptr,
    );

    UpacLib::check(
        unsafe { (machine.upac_lib.as_ref().list_commits)(list_request_c, &mut commit_array_c) },
        "list commits",
    )?;

    let commits_count =
        unsafe { (machine.upac_lib.as_ref().get_commits_count)(&mut commit_array_c) };
    let mut commit_rows = Vec::with_capacity(commits_count);

    for index in 0..commits_count {
        unsafe {
            let commit_handle_ptr = get_commit_handle!(
                machine.upac_lib.as_ref(),
                &mut commit_array_c,
                index as u8,
                "get commit handle"
            );

            let checksum = get_commit_field!(
                machine.upac_lib.as_ref(),
                commit_handle_ptr,
                0,
                "get checksum"
            );
            let subject = get_commit_field!(
                machine.upac_lib.as_ref(),
                commit_handle_ptr,
                1,
                "get subject"
            );

            commit_rows.push(CommitRow::new(checksum.to_owned(), subject.to_owned()));
        };
    }

    machine.commits = commit_rows;

    unsafe { (machine.upac_lib.as_ref().commits_free)(&mut commit_array_c) };

    state_printing_commits(machine)
}

fn state_get_packages_list(machine: &mut ListMachine) -> Result<()> {
    machine.state = State::GetPackages;

    let mut package_array_c: CArray<PackageMetaHandle> = CArray::empty();

    let token_ptr = cancel_token_ptr();

    let list_request_c = CUnmutatedRequest::for_list(
        &machine.config.paths.repo_path,
        &machine.config.paths.root_path,
        &machine.config.paths.database_path,
        &machine.config.ostree.branch,
        &machine.config.ostree.prefix_directory,
        token_ptr,
    );

    UpacLib::check(
        unsafe { (machine.upac_lib.as_ref().list_packages)(list_request_c, &mut package_array_c) },
        "list packages",
    )?;

    let package_count =
        unsafe { (machine.upac_lib.as_ref().get_packages_count)(&mut package_array_c) };
    let mut packages = Vec::with_capacity(package_count as usize);

    for index in 0..package_count {
        unsafe {
            let package_handle = get_package_handle!(
                machine.upac_lib.as_ref(),
                &mut package_array_c,
                index as u8,
                "get package handle"
            );

            let string_fields = [
                PackageField::Name,
                PackageField::Version,
                PackageField::Arch,
                PackageField::Author,
                PackageField::License,
                PackageField::Url,
                PackageField::Packager,
            ];

            let values = string_fields
                .iter()
                .map(|&f| {
                    Ok(get_package_field!(
                        machine.upac_lib.as_ref(),
                        package_handle,
                        f
                    ))
                })
                .collect::<Result<Vec<String>>>()?;

            let mut size_u64: u64 = 0;
            get_package_field!(machine.upac_lib.as_ref(), package_handle, PackageField::Size, @int size_u64)?;

            let mut it = values.into_iter();
            packages.push(PackageRow {
                name: it.next().unwrap(),
                version: it.next().unwrap(),
                architecture: it.next().unwrap(),
                author: it.next().unwrap(),
                license: it.next().unwrap(),
                url: it.next().unwrap(),
                packager: it.next().unwrap(),
                size: size_u64 as u32,
            });
        }
    }

    machine.packages = packages;
    unsafe { (machine.upac_lib.as_ref().packages_free)(&mut package_array_c) };

    state_printing_packeges(machine)
}

fn state_printing_commits(machine: &mut ListMachine) -> Result<()> {
    machine.state = State::PrintCommits;

    if machine.commits.is_empty() {
        println!("{}", "No commits found.".dimmed());
        return state_done(machine);
    }

    for row in &machine.commits {
        match machine.full {
            true => {
                println!("{}", &row.checksum[..12].bold().cyan());
                println!("  {} {}", "subject:".dimmed(), row.subject);
                println!("  {} {}", "hash:   ".dimmed(), row.checksum);
                println!();
            }
            false => {
                println!(
                    "{} {}",
                    &row.checksum[..12].bold().cyan(),
                    row.subject.dimmed()
                );
            }
        }
    }

    state_done(machine)
}

fn state_printing_packeges(machine: &mut ListMachine) -> Result<()> {
    machine.state = State::PrintPackages;

    if machine.packages.is_empty() {
        println!("{}", "No packages installed.".dimmed());
        return state_done(machine);
    }

    for package_row in &machine.packages {
        if machine.full {
            println!("{}", package_row.name.as_str().bold());
            println!(
                "  {} {}",
                "version: ".dimmed(),
                package_row.version.as_str()
            );
            println!("  {} {}", "size: ".dimmed(), format_size(package_row.size));
            println!(
                "  {} {}",
                "arch: ".dimmed(),
                package_row.architecture.as_str()
            );
            println!("  {} {}", "author: ".dimmed(), package_row.author.as_str());
            println!(
                "  {} {}",
                "packager: ".dimmed(),
                package_row.packager.as_str()
            );
            println!(
                "  {} {}",
                "license: ".dimmed(),
                package_row.license.as_str()
            );
            println!("  {} {}", "url: ".dimmed(), package_row.url.as_str());
            println!();
        } else {
            println!(
                "{} {}",
                package_row.name.as_str().bold(),
                package_row.version.as_str().dimmed()
            );
        }
    }

    state_done(machine)
}

fn state_done(machine: &mut ListMachine) -> Result<()> {
    machine.state = State::Done;

    Ok(())
}

// ── Helpers ───────────────────────────────────────────────────────────────────
pub fn format_size(bytes: u32) -> String {
    const KB: u32 = 1024;
    const MB: u32 = 1024 * KB;
    const GB: u32 = 1024 * MB;

    if bytes >= GB {
        format!("{:.1} GiB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MiB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KiB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}
