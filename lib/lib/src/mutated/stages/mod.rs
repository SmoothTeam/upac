// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::path::PathBuf;

use uuid::Uuid;

use upac_types::decoder::PackageTriggers;
use upac_types::diff::DiffFileSource;
use upac_types::package::PackageInfo;

use upac_composefs::Digest;
use upac_composefs::fs::WrittenFile;

use upac_decoder_loader::unpack::PackageTemp;

use upac_boot_loader::BootPlugin;

use upac_deploy::deployment::PrefixDeploy;
use upac_deploy::deployment::config::ConfigDeploy;

pub mod checkout;
pub mod commit;
pub mod deploy;
pub mod hooks;
pub mod merge;
pub mod open;
pub mod resolve;
pub mod retention;
pub mod swap;
pub mod unpack;

pub(crate) struct PackageSource {
    pub path: String,
    pub index: usize,
}

pub(crate) struct UnpackedPackage {
    pub temp: PackageTemp,
    pub triggers: PackageTriggers,
}

pub(crate) struct CommitInfo {
    pub subject: String,
    pub message: Option<String>,
    pub allow_conflict_files: bool,
}

pub(crate) struct RequestedBootPlugin(pub String);

pub(crate) struct RunningPrefix(pub PrefixDeploy);

pub(crate) struct NewConfig(pub ConfigDeploy);

pub(crate) struct DeployedPrefix {
    pub digest: Digest,
    pub next_written: WrittenFile,
}

pub(crate) struct BootTarget(pub Digest);

pub(crate) struct ResolvedBootEntry {
    pub plugin: BootPlugin,
    pub entry_name: String,
}

pub(crate) struct FileOwner(pub Uuid);

pub(crate) struct RequestedPackage(pub PackageInfo);

pub(crate) struct RequestedScope(pub DiffFileSource);

pub(crate) struct AttachItem {
    pub source: PathBuf,
    pub target: String,
}

pub(crate) struct DetachItem(pub String);
