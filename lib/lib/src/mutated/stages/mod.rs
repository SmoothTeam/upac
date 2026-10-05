// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::path::{Path, PathBuf};

use uuid::Uuid;

use upac_types::decoder::PackageTriggers;
use upac_types::diff::DiffFileSource;
use upac_types::error::ErrorKind;
use upac_types::package::PackageInfo;
use upac_types::response::entry::FileEntryScope;
use upac_types::transaction::Transaction;

use upac_composefs::Digest;
use upac_composefs::fs::WrittenFile;
use upac_composefs::tree::Tree;

use upac_database::MemoryDatabase;

use upac_decoder_loader::unpack::PackageTemp;

use upac_boot_loader::BootPlugin;

use upac_deploy::deployment::PrefixDeploy;
use upac_deploy::deployment::config::ConfigDeploy;

use crate::layout::prefix::{DEFAULTS_DIR, SYSTEM_CONFIG_ROOT, SYSTEM_PREFIX_ROOT};

pub mod checkout;
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

pub(crate) struct WorkingPrefix {
    pub tree: Tree,
    pub database: MemoryDatabase,
    pub parent_transaction: String,
}

pub(crate) struct NewPrefix {
    pub digest: Digest,
    pub transaction: Transaction,
}

pub(crate) struct NewDefaults(pub Tree);

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

pub(crate) struct ScopedPath {
    pub tree_path: PathBuf,
    pub entry_path: String,
    pub scope: FileEntryScope,
}

pub(crate) struct AttachItem {
    pub source: PathBuf,
    pub target: ScopedPath,
}

pub(crate) struct DetachItem(pub ScopedPath);

impl ScopedPath {
    pub fn new(scope: DiffFileSource, target: &str) -> Result<Self, ErrorKind> {
        let root = match scope {
            DiffFileSource::Prefix => SYSTEM_PREFIX_ROOT,
            DiffFileSource::Config => SYSTEM_CONFIG_ROOT,
        };

        let relative = Path::new(target)
            .strip_prefix(root)
            .map_err(|_| ErrorKind::InvalidPath)?;
        if relative.as_os_str().is_empty() {
            return Err(ErrorKind::InvalidPath);
        }

        let (tree_path, entry_scope) = match scope {
            DiffFileSource::Prefix => (relative.to_path_buf(), FileEntryScope::Prefix),
            DiffFileSource::Config => (Path::new(DEFAULTS_DIR).join(relative), FileEntryScope::Config),
        };

        Ok(Self {
            tree_path,
            entry_path: relative.to_string_lossy().into_owned(),
            scope: entry_scope,
        })
    }
}
