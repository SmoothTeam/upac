// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later

use std::fs::{create_dir_all, read_dir, remove_dir};
use std::path::{Path, PathBuf};

use composefs::repository::Repository;
use composefs::tree::FileSystem;

use nix::mount::{MsFlags, mount, umount};
use nix::sched::{CloneFlags, unshare};

use rsblkid::device::TagName;
use rsblkid::probe::Probe;
use rsblkid::utils::evaluation::find_canonical_device_name_from_path;

use rsmount::tables::MountInfo;

use self::error::SysrootError;

use crate::composefs::error::RepoError;
use crate::composefs::repository::{self, ObjectID};
use crate::layout::deployment::{DEPLOYS_DIR, NEXT_SEQ_PATH, REPO_DIR, ROOT_DIR, SYSROOT_DIR};

pub mod digest;
pub mod error;
pub mod esp;

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
    sysroot: PathBuf,
    deploy: PathBuf,
    repo: PathBuf,
}

impl Deploy {
    pub fn new(mode: DeployMode) -> Result<Self, SysrootError> {
        let device_path = Self::device_path()?;
        let filesystem_type = Self::filesystem_type(device_path.as_path())?;
        let sysroot = Self::sysroot_path()?;

        unshare(CloneFlags::CLONE_NEWNS)?;
        mount(
            None::<&str>,
            "/",
            None::<&str>,
            MsFlags::MS_REC | MsFlags::MS_PRIVATE,
            None::<&str>,
        )?;
        mount(
            Some(&device_path),
            &sysroot,
            Some(filesystem_type.as_str()),
            mode.into(),
            None::<&str>,
        )?;

        let deploy = sysroot.join(DEPLOYS_DIR);
        if !deploy.try_exists()? {
            return Err(SysrootError::DeploysDirNotFound);
        }

        let repo = sysroot.join(REPO_DIR);
        if !repo.try_exists()? {
            return Err(SysrootError::RepoDirNotFound);
        }

        Ok(Self { sysroot, deploy, repo })
    }

    pub fn deploy(&self, prefix_digest: &str) -> PathBuf {
        self.deploy.join(prefix_digest)
    }

    pub(crate) fn next_seq_path(&self) -> PathBuf {
        self.sysroot.join(NEXT_SEQ_PATH)
    }

    pub fn deploys(&self) -> Result<Vec<String>, SysrootError> {
        let mut digests = Vec::new();

        for entry in read_dir(&self.deploy)? {
            let entry = entry?;

            if !entry.file_type()?.is_dir() {
                continue;
            }

            if let Some(digest) = entry.file_name().to_str() {
                digests.push(digest.to_owned());
            }
        }

        Ok(digests)
    }

    pub fn repo(&self) -> &Path {
        &self.repo
    }

    pub fn open_repository(&self) -> Result<Repository<ObjectID>, RepoError> {
        repository::open(&self.repo)
    }

    pub fn open_tree(&self, name: &str) -> Result<FileSystem<ObjectID>, RepoError> {
        repository::open_tree(&self.open_repository()?, name)
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

    fn filesystem_type(device_path: &Path) -> Result<String, SysrootError> {
        let mut probe = Probe::builder()
            .scan_device(device_path)
            .scan_device_superblocks(true)
            .build()?;

        probe.find_device_properties();

        let tag = probe
            .lookup_device_property_value(TagName::Type)
            .ok_or(SysrootError::FilesystemTypeNotFound)?;

        Ok(tag.value().to_owned())
    }
}

impl Drop for Deploy {
    fn drop(&mut self) {
        let _ = umount(&self.sysroot);
        let _ = remove_dir(&self.sysroot);
    }
}
