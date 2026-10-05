// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_types::error::ErrorKind;
use upac_types::response::entry::ConfigCommitEntry;

use upac_composefs::Digest;

use upac_database::MemoryDatabase;

use upac_deploy::Sysroot;
use upac_deploy::deployment::config::ConfigDeploy;
use upac_deploy::deployment::{Deployment, PrefixDeploy};

pub(crate) mod diff;
pub(crate) mod diff_config;
pub(crate) mod diff_packages;
pub(crate) mod diff_prefix;

pub(crate) mod list_config;
pub(crate) mod list_history;
pub(crate) mod list_packages;
pub(crate) mod list_prefix;

pub(crate) mod search_files;
pub(crate) mod search_in_meta;
pub(crate) mod search_in_package_files;
pub(crate) mod search_meta;

pub(crate) struct RequestedPrefixDigestRange {
    pub from: Option<Digest>,
    pub to: Option<Digest>,
}

pub(crate) struct RequestedConfigDigestRange {
    pub from: Option<Digest>,
    pub to: Option<Digest>,
}

impl RequestedPrefixDigestRange {
    fn parse(from: Option<&str>, to: Option<&str>) -> Result<Self, ErrorKind> {
        Ok(Self {
            from: parse_digest(from)?,
            to: parse_digest(to)?,
        })
    }
}

impl RequestedConfigDigestRange {
    fn parse(from: Option<&str>, to: Option<&str>) -> Result<Self, ErrorKind> {
        Ok(Self {
            from: parse_digest(from)?,
            to: parse_digest(to)?,
        })
    }
}

fn parse_digest(hex: Option<&str>) -> Result<Option<Digest>, ErrorKind> {
    Ok(hex.map(Digest::from_hex).transpose()?)
}

fn requested_prefix(sysroot: &Sysroot, prefix_digest: Option<&Digest>) -> Result<PrefixDeploy, ErrorKind> {
    match prefix_digest {
        Some(prefix_digest) => Ok(sysroot.prefix(prefix_digest)?),
        None => Ok(sysroot.running_prefix()?),
    }
}

fn running_prefix_database(sysroot: &Sysroot) -> Result<MemoryDatabase, ErrorKind> {
    Ok(sysroot.prefix_database(sysroot.running_prefix()?.digest())?)
}

fn requested_prefix_config<'prefix>(
    prefix: &'prefix PrefixDeploy, config_digest: Option<&Digest>,
) -> Result<&'prefix ConfigDeploy, ErrorKind> {
    match config_digest {
        Some(config_digest) => prefix.configs().iter().find(|config| config.digest() == config_digest),
        None => prefix.current_config(),
    }
    .ok_or(ErrorKind::NotFound)
}

fn config_commit_entry(config: &ConfigDeploy) -> ConfigCommitEntry {
    ConfigCommitEntry {
        config_digest: config.digest().to_hex(),
        subject: config.subject().to_owned(),
        message: config.message().map(str::to_owned),
    }
}
