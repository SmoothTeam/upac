// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::fs::{File, create_dir, create_dir_all, read_dir, remove_dir_all, remove_file};
use std::io::ErrorKind as IoErrorKind;
use std::path::{Path, PathBuf};

use upac_composefs::Digest;

use super::Sysroot;
use super::error::DeployError;
use super::layout::deployment::{CONFIG_DIR, HISTORY_DIR, PINNED_FILE, UPPER_DIR, WORK_DIR};
use super::snapshot::ConfigSnapshot;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SystemDeploy {
    digest: Digest,
    dir: PathBuf,
}

impl SystemDeploy {
    pub fn create(sysroot: &Sysroot, digest: Digest) -> Result<Self, DeployError> {
        fn new(sysroot: &Sysroot, digest: Digest) -> SystemDeploy {
            SystemDeploy {
                dir: sysroot.deploys_dir.join(digest.to_hex()),
                digest,
            }
        }

        let deploy = new(sysroot, digest);

        if deploy.dir.try_exists()? {
            return Err(DeployError::AlreadyExists);
        }

        create_dir(&deploy.dir)?;
        if let Err(error) = create_dir_all(deploy.upper_dir())
            .and_then(|()| create_dir(deploy.work_dir()))
            .and_then(|()| create_dir(deploy.dir.join(HISTORY_DIR)))
        {
            let _ = remove_dir_all(&deploy.dir);
            return Err(error.into());
        }

        Ok(deploy)
    }

    pub fn open(sysroot: &Sysroot, digest: Digest) -> Result<Self, DeployError> {
        fn new(sysroot: &Sysroot, digest: Digest) -> SystemDeploy {
            SystemDeploy {
                dir: sysroot.deploys_dir.join(digest.to_hex()),
                digest,
            }
        }

        let deploy = new(sysroot, digest);

        if !deploy.dir.try_exists()? {
            return Err(DeployError::NotFound);
        }

        Ok(deploy)
    }

    pub fn remove(self) -> Result<(), DeployError> {
        Ok(remove_dir_all(self.dir)?)
    }

    pub fn digest(&self) -> &Digest {
        &self.digest
    }

    pub fn upper_dir(&self) -> PathBuf {
        self.dir.join(CONFIG_DIR).join(UPPER_DIR)
    }

    pub fn work_dir(&self) -> PathBuf {
        self.dir.join(CONFIG_DIR).join(WORK_DIR)
    }

    pub fn is_pinned(&self) -> Result<bool, DeployError> {
        Ok(self.dir.join(PINNED_FILE).try_exists()?)
    }

    pub fn set_pinned(&self, pinned: bool) -> Result<(), DeployError> {
        let pinned_file = self.dir.join(PINNED_FILE);

        if pinned {
            File::create(pinned_file)?;
        } else {
            match remove_file(pinned_file) {
                Ok(()) => {}
                Err(error) if error.kind() == IoErrorKind::NotFound => {}
                Err(error) => return Err(error.into()),
            }
        }

        Ok(())
    }

    pub fn snapshots(&self) -> Result<Vec<ConfigSnapshot>, DeployError> {
        let mut snapshots = Vec::new();
        for record in read_dir(self.history_dir())? {
            snapshots.push(ConfigSnapshot::read(self, &record?.file_name())?);
        }

        snapshots.sort_by_key(ConfigSnapshot::timestamp);

        Ok(snapshots)
    }

    pub fn add_snapshot(&self, snapshot: &ConfigSnapshot) -> Result<(), DeployError> {
        snapshot.write(self)
    }

    pub(crate) fn dir(&self) -> &Path {
        &self.dir
    }

    pub(crate) fn history_dir(&self) -> PathBuf {
        self.dir.join(HISTORY_DIR)
    }
}
