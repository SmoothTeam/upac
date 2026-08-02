// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 JustPav
//
// SPDX-License-Identifier: LGPL-3.0-or-later

use std::fs::{create_dir_all, remove_dir};
use std::path::{Path, PathBuf};

use nix::mount::{MsFlags, mount, umount};
use nix::sched::{CloneFlags, unshare};
use rsblkid::cache::Cache;
use rsblkid::device::TagName;
use rsblkid::utils::evaluation::find_canonical_device_name_from_path;
use rsmount::tables::MountInfo;
use uuid::Uuid;

pub use self::error::SysrootError;

use crate::types::deployment::{DEPLOYS_DIR, REPO_DIR, ROOT_DIR, SYSROOT_DIR};

mod error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeployMode {
    ReadOnly,
    ReadWrite,
}

impl From<DeployMode> for MsFlags {
    fn from(mode: DeployMode) -> Self {
        match mode {
            DeployMode::ReadOnly => MsFlags::MS_RDONLY,
            DeployMode::ReadWrite => MsFlags::empty(),
        }
    }
}

pub struct Deploy {
    uuid: Uuid,

    sysroot: PathBuf,
    deploy: PathBuf,
    repo: PathBuf,
}

impl Deploy {
    pub fn new(mode: DeployMode) -> Result<Self, SysrootError> {
        let device_path = Self::device_path()?;

        let mut cache = Cache::builder().discard_changes_on_drop().build()?;
        cache.add_new_entry(&device_path)?;

        let uuid = Self::uuid(&cache, device_path.as_path())?;
        let sysroot = Self::sysroot_path()?;

        unshare(CloneFlags::CLONE_NEWNS)?;
        mount(Some(&device_path), &sysroot, None::<&str>, mode.into(), None::<&str>)?;

        let deploy = sysroot.join(DEPLOYS_DIR);
        if !deploy.try_exists()? {
            return Err(SysrootError::DeploysDirNotFound);
        }

        let repo = sysroot.join(REPO_DIR);
        if !repo.try_exists()? {
            return Err(SysrootError::RepoDirNotFound);
        }

        Ok(Self {
            uuid,
            sysroot,
            deploy,
            repo,
        })
    }

    pub fn deploy(&self, os: &str, checksum: &str) -> PathBuf {
        self.deploy.join(os).join(checksum)
    }

    pub fn repo(&self) -> &Path {
        &self.repo
    }

    fn sysroot_path() -> Result<PathBuf, SysrootError> {
        let sysroot = Path::new(ROOT_DIR).join(SYSROOT_DIR);
        create_dir_all(&sysroot)?;

        Ok(sysroot)
    }

    fn device_path() -> Result<PathBuf, SysrootError> {
        let mut table = MountInfo::new()?;
        table.import_mountinfo()?;

        let raw_device_path = table
            .find_target(ROOT_DIR)
            .and_then(|entry| entry.source_path())
            .ok_or(SysrootError::RootDeviceNotFound)?;

        find_canonical_device_name_from_path(raw_device_path).ok_or(SysrootError::CanonicalDeviceNotFound)
    }

    fn uuid(cache: &Cache, device_path: &Path) -> Result<Uuid, SysrootError> {
        let raw_uuid_as_raw_bytes = cache
            .tag_value_from_device(TagName::Uuid, device_path)
            .ok_or(SysrootError::UuidNotFound)?;

        Ok(Uuid::parse_str(raw_uuid_as_raw_bytes.as_str_lossy())?)
    }
}

impl Drop for Deploy {
    fn drop(&mut self) {
        let _ = umount(&self.sysroot);
        let _ = remove_dir(&self.sysroot);
    }
}
