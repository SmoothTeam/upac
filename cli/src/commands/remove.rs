// ── Imports ─────────────────────────────────────────────────────────────────
use anyhow::Result;
use colored::Colorize;
use indicatif::ProgressBar;

use std::ffi::{c_void, CString};
use std::sync::Arc;

use crate::cancel_token_ptr;
use crate::config::Config;
use crate::ffi::{CMutatedRequest, CSlice};
use crate::upac::UpacLib;
use crate::utils::spinner;

// ── Arguments for command ───────────────────────────────────────────────────────────────────────
#[derive(clap::Args)]
pub struct RemoveArgs {
    #[arg(required = true, num_args = 1..)]
    pub name: Vec<CString>,
}

// ── FSM states ───────────────────────────────────────────────────────────────────────
#[derive(Debug, Clone, PartialEq)]
enum State {
    Validating,
    Uninstalling,
    Done,
}

// ── FSM machine ────────────────────────────────────────────────────────────────────────
struct RemoveMachine {
    package_names: Vec<CString>,

    upac_lib: Arc<UpacLib>,
    progress_bar: ProgressBar,
    config: Config,
    state: State,
}

impl RemoveMachine {
    fn new(config: Config, upac_lib: Arc<UpacLib>, package_names: Vec<CString>) -> Result<Self> {
        Ok(Self {
            package_names,
            progress_bar: ProgressBar::new_spinner(),
            upac_lib,
            config,
            state: State::Validating,
        })
    }
}

// ── Public API ─────────────────────────────────────────────────────────────
pub fn run(args: RemoveArgs, config: Config, upac_lib: Arc<UpacLib>) -> Result<()> {
    let mut remove_machine = RemoveMachine::new(config, upac_lib, args.name)?;

    state_validating(&mut remove_machine).inspect_err(|_| {
        if remove_machine.config.verbose {
            eprintln!(
                "{} failed at state {:?}",
                "✗".red().bold(),
                remove_machine.state
            );
        }
    })
}

// ── States ─────────────────────────────────────────────────────────────────
fn state_validating(machine: &mut RemoveMachine) -> Result<()> {
    machine.state = State::Validating;
    spinner(&machine.progress_bar, "Removing packages...");

    for name in &machine.package_names {
        if name.is_empty() {
            anyhow::bail!("Package name cannot be empty");
        }
        machine
            .progress_bar
            .println(format!("{} removing {}", "→".cyan(), name.to_str()?.bold()));
    }

    state_uninstalling(machine)
}

fn state_uninstalling(machine: &mut RemoveMachine) -> Result<()> {
    machine.state = State::Uninstalling;

    let package_names_c: Vec<CSlice> = machine
        .package_names
        .iter()
        .map(CSlice::from_cstring)
        .collect();

    let progress_bar_ptr = &machine.progress_bar as *const ProgressBar as *mut c_void;

    let token_ptr = cancel_token_ptr();

    let remove_request_c = CMutatedRequest::for_uninstall(
        package_names_c.as_slice(),
        &machine.config.paths.repo_path,
        &machine.config.paths.root_path,
        &machine.config.ostree.branch,
        &machine.config.ostree.prefix_directory,
        machine.config.step_retries,
        Some(on_remove_progress),
        progress_bar_ptr,
        token_ptr,
    );

    UpacLib::check(
        unsafe { (machine.upac_lib.as_ref().uninstall)(remove_request_c) },
        "uninstall",
    )
    .inspect_err(|_| {
        machine.progress_bar.finish_and_clear();
    })?;

    state_done(machine)
}

fn state_done(machine: &mut RemoveMachine) -> Result<()> {
    machine.state = State::Done;
    machine.progress_bar.finish_and_clear();

    for name in &machine.package_names {
        machine.progress_bar.println(format!(
            "{} removed {}",
            "✓".green().bold(),
            name.to_str()?.bold()
        ));
    }

    Ok(())
}

// ── Helpers ───────────────────────────────────────────────────────────────────
pub unsafe extern "C" fn on_remove_progress(event: u32, package_name: CSlice, ctx: *mut c_void) {
    let progress_bar = &*(ctx as *const ProgressBar);

    let name = unsafe { package_name.as_str() };

    match event {
        0 => progress_bar.set_message(format!("Verifying {name}...")),
        1 => progress_bar.set_message("Opening repo...".to_string()),
        2 => progress_bar.set_message(format!("Checking {name} installed...")),
        3 => {}
        4 => progress_bar.set_message(format!("Removing files for {name}...")),
        5 => progress_bar.set_message(format!("Removing database for {name}...")),
        6 => progress_bar.set_message(format!("Committing {name}...")),
        7 => progress_bar.set_message(format!("Cheking out {name}...")),
        8 => {}
        9 => {}
        10 => progress_bar.println(format!("{} Done", "✓".green().bold())),
        11 => progress_bar.println(format!("{} Failed", "✗".red().bold())),
        _ => {
            eprintln!("Unknow event: {}", event);
        }
    }
}
