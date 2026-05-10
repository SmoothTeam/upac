// ── Imports ─────────────────────────────────────────────────────────────────
use anyhow::Result;

use colored::Colorize;

use indicatif::ProgressBar;

use sha2::{Digest, Sha256};

use std::collections::HashMap;
use std::env;
use std::ffi::c_void;
use std::ffi::CString;
use std::fs;
use std::io::Read;
use std::ptr::{null, null_mut};
use std::str::FromStr;
use std::sync::Arc;

use crate::backends::Backend;
use crate::cancel_token_ptr;
use crate::config::Config;
use crate::ffi::{CMutatedRequest, CPackageEntry, CSlice, PackageMetaHandle};
use crate::upac::UpacLib;
use crate::utils::{spinner, BackendKind};

// ── Arguments for command ───────────────────────────────────────────────────────────────────────
#[derive(clap::Args)]
pub struct InstallArgs {
    #[arg(required = true, num_args = 1..)]
    pub files: Vec<String>,
    #[arg(long)]
    pub backend: Option<String>,
    #[arg(long, num_args = 0..)]
    pub checksums: Vec<String>,
}

// ── FSM states ───────────────────────────────────────────────────────────────────────
#[derive(Debug, Clone, PartialEq)]
enum State {
    DetectingBackend,
    PreparingPackage,
    Installing,
    Done,
}

pub struct PreparedPackage {
    pub meta_handle: PackageMetaHandle,
    pub temp_path_c: CSlice,
    pub checksum: CString,
    pub backend: Arc<Backend>,
}

impl PreparedPackage {
    pub fn new(
        meta_handle: PackageMetaHandle,
        temp_path_c: CSlice,
        checksum: CString,
        backend: Arc<Backend>,
    ) -> Self {
        Self {
            meta_handle,
            temp_path_c,
            checksum,
            backend,
        }
    }

    pub fn as_c_entry(&self) -> CPackageEntry {
        CPackageEntry::new(
            self.meta_handle,
            self.temp_path_c,
            CSlice::from_cstring(&self.checksum),
        )
    }
}

impl Drop for PreparedPackage {
    fn drop(&mut self) {
        unsafe {
            if !self.meta_handle.is_null() {
                (self.backend.meta_free)(self.meta_handle);
                self.meta_handle = null_mut();
            }
            if !self.temp_path_c.ptr.is_null() {
                (self.backend.cleanup)(self.temp_path_c);
                self.temp_path_c = CSlice {
                    ptr: null(),
                    len: 0,
                };
            }
        }
    }
}

// ── FSM machine ───────────────────────────────────────────────────────────────────────
struct InstallMachine {
    files: Vec<String>,
    backend: Option<String>,
    checksums: Vec<String>,

    prepared_packages: Vec<PreparedPackage>,

    upac_lib: Arc<UpacLib>,
    loaded_backends: HashMap<BackendKind, Arc<Backend>>,
    progress_bar: ProgressBar,
    config: Config,
    state: State,
}

impl InstallMachine {
    fn new(
        config: Config,
        upac_lib: Arc<UpacLib>,
        files: Vec<String>,
        backend: Option<String>,
        checksums: Vec<String>,
    ) -> Result<Self> {
        Ok(Self {
            files,
            backend,
            checksums,
            prepared_packages: Vec::new(),
            progress_bar: ProgressBar::new_spinner(),
            upac_lib,
            loaded_backends: HashMap::new(),
            config,
            state: State::PreparingPackage,
        })
    }
}

// ── Public API ─────────────────────────────────────────────────────────────
pub fn run(args: InstallArgs, config: Config, upac_lib: Arc<UpacLib>) -> Result<()> {
    if !args.checksums.is_empty() && args.checksums.len() != args.files.len() {
        anyhow::bail!(
            "Count of checksums ({}) must match count of files ({})",
            args.checksums.len(),
            args.files.len()
        );
    }

    let mut install_machine =
        InstallMachine::new(config, upac_lib, args.files, args.backend, args.checksums)?;

    state_preparing_package(&mut install_machine).inspect_err(|_| {
        if install_machine.config.verbose {
            eprintln!(
                "{} failed at state {:?}",
                "✗".red().bold(),
                install_machine.state
            );
        }
    })
}

// ── States ─────────────────────────────────────────────────────────────────
fn state_preparing_package(machine: &mut InstallMachine) -> Result<()> {
    machine.state = State::PreparingPackage;
    spinner(&machine.progress_bar, "Verifying and extracting package...");

    let files: Vec<String> = machine.files.clone();

    let progress_bar_ptr = &machine.progress_bar as *const ProgressBar as *mut c_void;

    for (index, file) in files.iter().enumerate() {
        machine.state = State::DetectingBackend;

        let kind = if let Some(flag) = &machine.backend {
            BackendKind::from_flag(flag)?
        } else {
            BackendKind::detect(file).ok_or_else(|| {
                anyhow::anyhow!("Cannot detect backend for '{file}'. Use --backend to specify one")
            })?
        };

        let backend = machine
            .loaded_backends
            .entry(kind.clone())
            .or_insert_with(|| {
                Arc::new(
                    Backend::load(&kind)
                        .unwrap_or_else(|_| panic!("Failed to load backend lib for {}", kind)),
                )
            })
            .clone();

        machine
            .progress_bar
            .println(format!("{} Backend: {}", "→".cyan(), kind));

        let tmp_string_path = CString::from_str(
            env::temp_dir()
                .to_str()
                .ok_or_else(|| anyhow::anyhow!("invalid temp dir path"))?,
        )?;

        let abs_file_path = fs::canonicalize(file)
            .map_err(|err| anyhow::anyhow!("cannot resolve path '{file}': {err}"))?;
        let abs_file_str = abs_file_path
            .to_str()
            .ok_or_else(|| anyhow::anyhow!("invalid path encoding"))?;

        let checksum = if machine.checksums.is_empty() {
            CString::from_str(&compute_checksum(abs_file_str)?)?
        } else {
            CString::from_str(&machine.checksums[index])?
        };

        let (meta_handle, temp_path_c) = backend
            .meta_prepare(
                &CString::from_str(abs_file_str)?,
                &tmp_string_path,
                &checksum,
                progress_bar_ptr,
                crate::cancel_token_ptr(),
            )
            .inspect_err(|_| {
                machine.progress_bar.finish_and_clear();
            })?;

        let prepared_packege =
            PreparedPackage::new(meta_handle, temp_path_c, checksum.to_owned(), backend);

        machine.prepared_packages.push(prepared_packege);
    }

    state_installing(machine)
}

fn state_installing(machine: &mut InstallMachine) -> Result<()> {
    machine.state = State::Installing;

    let progress_bar_ptr = &machine.progress_bar as *const ProgressBar as *mut c_void;

    let packages_c: Vec<CPackageEntry> = machine
        .prepared_packages
        .iter()
        .map(|prepared_package| prepared_package.as_c_entry())
        .collect();

    let token_ptr = cancel_token_ptr();

    let install_request_c = CMutatedRequest::for_install(
        packages_c.as_slice(),
        &machine.config.paths.repo_path,
        &machine.config.paths.root_path,
        &machine.config.ostree.branch,
        machine.config.step_retries,
        Some(on_install_progress),
        progress_bar_ptr,
        token_ptr,
    );

    UpacLib::check(
        unsafe { (machine.upac_lib.as_ref().install)(install_request_c) },
        "install",
    )
    .inspect_err(|_| {
        machine.progress_bar.finish_and_clear();
    })?;

    state_done(machine)
}

fn state_done(machine: &mut InstallMachine) -> Result<()> {
    machine.state = State::Done;
    machine.progress_bar.finish_and_clear();

    for package in &machine.prepared_packages {
        let backend = &package.backend;
        let name = unsafe {
            (backend.meta_get_name)(package.meta_handle)
                .as_str()
                .to_owned()
        };
        let version = unsafe {
            (backend.meta_get_version)(package.meta_handle)
                .as_str()
                .to_owned()
        };

        machine.progress_bar.println(format!(
            "{} installed {} {}",
            "✓".green().bold(),
            name.bold(),
            version
        ));
    }

    machine.prepared_packages.clear();
    machine.loaded_backends.clear();

    Ok(())
}

// ── Helpers ───────────────────────────────────────────────────────────────────
pub unsafe extern "C" fn on_install_progress(event: u32, package_name_c: CSlice, ctx: *mut c_void) {
    let progress_bar = &*(ctx as *const ProgressBar);

    let package_name = unsafe { package_name_c.as_str() };

    match event {
        0 => progress_bar.set_message(format!("Verifying {package_name}...")),
        1 => progress_bar.set_message(format!("Checking free space for {package_name}...")),
        2 => progress_bar.set_message("Opening repo...".to_string()),
        3 => progress_bar.set_message(format!("Checking {package_name} was installed...")),
        4 => progress_bar.set_message(format!("Writing database for {package_name}...")),
        5 => {}
        6 => progress_bar.set_message(format!("Writing files for {package_name}...")),
        7 => progress_bar.set_message(format!("Committing {package_name}...")),
        8 => progress_bar.set_message(format!("Checking out {package_name}...")),
        9 => progress_bar.set_message("Preparing config files...".to_string()),
        10 => {}
        11 => {}
        12 => {}

        13 => progress_bar.println(format!("{} Done", "✓".green().bold())),
        14 => progress_bar.println(format!("{} Failed", "✗".red().bold())),
        _ => {
            eprintln!("Unknown event: {}", event);
        }
    }
}

fn compute_checksum(file_path: &str) -> Result<String> {
    let mut file =
        fs::File::open(file_path).map_err(|err| anyhow::anyhow!("failed to open file: {err}"))?;

    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 4096];

    loop {
        let bytes_read = file.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    Ok(hex::encode(hasher.finalize()))
}
