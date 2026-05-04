// ── Imports ─────────────────────────────────────────────────────────────────
use anyhow::Result;

use indicatif::ProgressBar;

use colored::Colorize;

use std::ffi::CString;
use std::sync::Arc;

use crate::config::Config;
use crate::ffi::{CMutatedRequest, CancelToken};
use crate::upac::UpacLib;
use crate::utils::{spinner, BackendKind};

// ── Arguments for command ───────────────────────────────────────────────────────────────────────
#[derive(clap::Args)]
pub struct RollbackArgs {
    pub commit: CString,
}

// ── FSM states ───────────────────────────────────────────────────────────────────────
#[derive(Debug, Clone, PartialEq)]
enum State {
    Validating,
    RollingBack,
    Done,
}

// ── FSM machine ───────────────────────────────────────────────────────────────────────
struct RollbackMachine {
    commit_hash: CString,

    upac_lib: Arc<UpacLib>,
    progress_bar: ProgressBar,
    config: Config,
    state: State,
}

impl RollbackMachine {
    fn new(config: Config, commit_hash: CString) -> Result<Self> {
        Ok(Self {
            commit_hash,
            progress_bar: ProgressBar::new_spinner(),
            upac_lib: Arc::new(UpacLib::load(&BackendKind::UpacLib)?),
            config,
            state: State::Validating,
        })
    }
}

// ── Public API ─────────────────────────────────────────────────────────────
pub fn run(config: Config, args: RollbackArgs) -> Result<()> {
    let mut rolling_machine = RollbackMachine::new(config, args.commit)?;

    state_validating(&mut rolling_machine).map_err(|err| {
        if rolling_machine.config.verbose {
            eprintln!(
                "{} failed at state {:?}",
                "✗".red().bold(),
                rolling_machine.state
            );
        }
        err
    })
}

// ── States ─────────────────────────────────────────────────────────────────
fn state_validating(machine: &mut RollbackMachine) -> Result<()> {
    machine.state = State::Validating;
    spinner(&machine.progress_bar, "Validating rolling data...");

    if machine.commit_hash.as_bytes().len() != 64
        || !machine
            .commit_hash
            .as_bytes()
            .iter()
            .all(|byte| byte.is_ascii_hexdigit())
    {
        anyhow::bail!(
            "invalid commit hash '{:?}'. Expected 64 hex characters",
            machine.commit_hash
        );
    }

    machine.progress_bar.println(format!(
        "{} rolling back to {}",
        "→".cyan(),
        &machine.commit_hash.to_str()?[..12].dimmed()
    ));

    state_rolling_back(machine)
}

fn state_rolling_back(machine: &mut RollbackMachine) -> Result<()> {
    machine.state = State::RollingBack;
    spinner(&machine.progress_bar, "Rolling back...");

    let mut token = Box::new(CancelToken::zeroed());
    let token_ptr = &mut *token as *mut CancelToken;

    let rollback_request_c = CMutatedRequest::for_rollback(
        &machine.commit_hash,
        &machine.config.paths.repo_path,
        &machine.config.paths.root_path,
        &machine.config.paths.database_path,
        &machine.config.ostree.branch,
        &machine.config.ostree.prefix_directory,
        machine.config.step_retries,
        token_ptr,
    );

    UpacLib::check(
        unsafe { (machine.upac_lib.as_ref().rollback)(rollback_request_c) },
        "rollback",
    )?;

    state_done(machine)
}

fn state_done(machine: &mut RollbackMachine) -> Result<()> {
    machine.state = State::Done;
    machine.progress_bar.finish_and_clear();

    println!(
        "{} rolled back to {}",
        "✓".green().bold(),
        &machine.commit_hash.to_str()?[..12].bold()
    );

    Ok(())
}
