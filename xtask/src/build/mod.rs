// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: GPL-3.0-only

use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

use clap::{Args as ClapArgs, ValueEnum};

use crate::error::XtaskError;

#[derive(Clone, Copy, ValueEnum)]
enum Arch {
    #[value(name = "x86-64-v1")]
    X86_64V1,
    #[value(name = "x86-64-v2")]
    X86_64V2,
    #[value(name = "x86-64-v3")]
    X86_64V3,
    #[value(name = "x86-64-v4")]
    X86_64V4,
}

impl Arch {
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

#[derive(Clone, Copy, ValueEnum)]
enum LinkMode {
    #[value(name = "dynamic")]
    Dynamic,
    #[value(name = "lib-static")]
    LibStatic,
    #[value(name = "full-static")]
    FullStatic,
}

#[derive(Clone, Copy, ValueEnum)]
enum Component {
    #[value(name = "uki")]
    Uki,
    #[value(name = "systemd-boot")]
    SystemdBoot,
    #[value(name = "grub")]
    Grub,
}

impl Component {
    fn feature(&self) -> &'static str {
        match self {
            Component::Uki => "upac-lib/static-uki",
            Component::SystemdBoot => "upac-lib/static-systemd-boot",
            Component::Grub => "upac-lib/static-grub",
        }
    }

    fn package_name(&self) -> &'static str {
        match self {
            Component::Uki => "upac-uki",
            Component::SystemdBoot => "upac-systemd-boot",
            Component::Grub => "upac-grub",
        }
    }
}

#[derive(ClapArgs)]
pub struct Args {
    /// Target microarchitecture level to compile for
    #[arg(long)]
    arch: Arch,
    /// How to link upac-cli against upac-lib and its boot-plugin components
    #[arg(long, default_value = "dynamic")]
    link: LinkMode,
    /// Boot-plugin components to statically embed (requires --link lib-static or full-static)
    #[arg(long, value_delimiter = ',')]
    components: Vec<Component>,
}

impl Args {
    fn validate(&self) -> Result<(), XtaskError> {
        if !self.components.is_empty() && matches!(self.link, LinkMode::Dynamic) {
            return Err(XtaskError::ComponentsRequireStaticLink);
        }

        Ok(())
    }
}

pub fn run(args: Args) -> Result<ExitCode, XtaskError> {
    args.validate()?;

    let repo_root = repo_root()?;

    let mut command = Command::new("cargo");
    command
        .current_dir(&repo_root)
        .args(["build", "--workspace", "--profile", args.arch.profile_name()])
        .env("RUSTFLAGS", format!("-C target-cpu={}", args.arch.target_cpu()));

    if !matches!(args.link, LinkMode::Dynamic) {
        let mut features = Vec::new();
        if let LinkMode::FullStatic = args.link {
            features.push("upac-cli/static-link".to_owned());
        }
        features.extend(args.components.iter().map(|component| component.feature().to_owned()));

        if !features.is_empty() {
            command.args(["--features", &features.join(",")]);
        }
        for component in &args.components {
            command.args(["--exclude", component.package_name()]);
        }
    }

    let status = command.status()?;

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
