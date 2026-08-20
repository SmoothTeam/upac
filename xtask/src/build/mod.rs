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

enum LinkMode {
    Dynamic,
    LibStatic,
    FullStatic,
}

impl LinkMode {
    fn parse(value: &str) -> Result<Self, XtaskError> {
        match value {
            "dynamic" => Ok(LinkMode::Dynamic),
            "lib-static" => Ok(LinkMode::LibStatic),
            "full-static" => Ok(LinkMode::FullStatic),
            other => Err(XtaskError::InvalidLinkMode(other.to_owned())),
        }
    }
}

enum Component {
    Uki,
    SystemdBoot,
}

impl Component {
    fn parse(value: &str) -> Result<Self, XtaskError> {
        match value {
            "uki" => Ok(Component::Uki),
            "systemd-boot" => Ok(Component::SystemdBoot),
            other => Err(XtaskError::InvalidComponent(other.to_owned())),
        }
    }

    fn feature(&self) -> &'static str {
        match self {
            Component::Uki => "upac-lib/static-uki",
            Component::SystemdBoot => "upac-lib/static-systemd-boot",
        }
    }
}

pub struct Args {
    arch: Arch,
    link: LinkMode,
    components: Vec<Component>,
}

impl Args {
    pub fn parse(raw: &[String]) -> Result<Self, XtaskError> {
        let mut arch = None;
        let mut link = LinkMode::Dynamic;
        let mut components = Vec::new();

        let mut iter = raw.iter();
        while let Some(arg) = iter.next() {
            match arg.as_str() {
                "--arch" => {
                    let value = iter.next().ok_or(XtaskError::MissingArchValue)?;
                    arch = Some(Arch::parse(value)?);
                }
                "--link" => {
                    let value = iter.next().ok_or(XtaskError::MissingLinkValue)?;
                    link = LinkMode::parse(value)?;
                }
                "--components" => {
                    let value = iter.next().ok_or(XtaskError::MissingLinkValue)?;
                    for name in value.split(',') {
                        components.push(Component::parse(name)?);
                    }
                }
                other => return Err(XtaskError::UnknownArgument(other.to_owned())),
            }
        }

        if !components.is_empty() && matches!(link, LinkMode::Dynamic) {
            return Err(XtaskError::ComponentsRequireStaticLink);
        }

        Ok(Self {
            arch: arch.ok_or(XtaskError::MissingArchValue)?,
            link,
            components,
        })
    }
}

pub fn run(args: Args) -> Result<ExitCode, XtaskError> {
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
        if !args.components.is_empty() {
            command.args(["--exclude", "upac-uki", "--exclude", "upac-systemd-boot"]);
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
