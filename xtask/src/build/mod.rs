// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: GPL-3.0-only

use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

use crate::error::XtaskError;

enum Arch {
    X86_64V1,
    X86_64V2,
    X86_64V3,
    X86_64V4,
}

impl Arch {
    fn parse(value: &str) -> Result<Self, XtaskError> {
        match value {
            "x86-64-v1" => Ok(Arch::X86_64V1),
            "x86-64-v2" => Ok(Arch::X86_64V2),
            "x86-64-v3" => Ok(Arch::X86_64V3),
            "x86-64-v4" => Ok(Arch::X86_64V4),
            other => Err(XtaskError::InvalidArch(other.to_owned())),
        }
    }

    fn target_cpu(&self) -> &'static str {
        match self {
            Arch::X86_64V1 => "x86-64",
            Arch::X86_64V2 => "x86-64-v2",
            Arch::X86_64V3 => "x86-64-v3",
            Arch::X86_64V4 => "x86-64-v4",
        }
    }

    fn profile_name(&self) -> &'static str {
        match self {
            Arch::X86_64V1 => "release-x86-64-v1",
            Arch::X86_64V2 => "release-x86-64-v2",
            Arch::X86_64V3 => "release-x86-64-v3",
            Arch::X86_64V4 => "release-x86-64-v4",
        }
    }
}

pub struct Args {
    arch: Arch,
}

impl Args {
    pub fn parse(raw: &[String]) -> Result<Self, XtaskError> {
        let mut arch = None;

        let mut iter = raw.iter();
        while let Some(arg) = iter.next() {
            match arg.as_str() {
                "--arch" => {
                    let value = iter.next().ok_or(XtaskError::MissingArchValue)?;
                    arch = Some(Arch::parse(value)?);
                }
                other => return Err(XtaskError::UnknownArgument(other.to_owned())),
            }
        }

        Ok(Self {
            arch: arch.ok_or(XtaskError::MissingArchValue)?,
        })
    }
}

pub fn run(args: Args) -> Result<ExitCode, XtaskError> {
    let repo_root = repo_root()?;

    let status = Command::new("cargo")
        .current_dir(&repo_root)
        .args(["build", "--workspace", "--profile", args.arch.profile_name()])
        .env("RUSTFLAGS", format!("-C target-cpu={}", args.arch.target_cpu()))
        .status()?;

    Ok(if status.success() {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    })
}

fn repo_root() -> Result<PathBuf, XtaskError> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    manifest_dir
        .parent()
        .map(Path::to_path_buf)
        .ok_or(XtaskError::RepoRootNotFound)
}
