// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::fs::{DirEntry, Permissions, create_dir_all, read_dir, read_link, remove_file, rename, set_permissions};
use std::io::{Error as IoError, ErrorKind as IoErrorKind};
use std::os::unix::fs::{PermissionsExt, symlink};
use std::path::{Path, PathBuf};

use nix::mount::{MsFlags, mount};
use nix::sched::{CloneFlags, unshare};

use procfs::mounts;

use upac_composefs::{Digest, Repo};

use self::deployment::SystemDeploy;
use self::error::SysrootError;
use self::layout::deployment::{
    CURRENT_LINK, CURRENT_LINK_STAGING, DEPLOYS_DIR, REPO_DIR, ROOT_DIR, SYSROOT_DIR, SYSROOT_ROOT_MODE,
};

pub mod deployment;
pub mod error;
pub mod fs;
pub mod layout {
    include!(concat!(env!("OUT_DIR"), "/layout.rs"));
}
pub mod snapshot;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SysrootMode {
    ReadOnly,
    ReadWrite,
}

impl From<SysrootMode> for MsFlags {
    fn from(mode: SysrootMode) -> Self {
        match mode {
            SysrootMode::ReadOnly => MsFlags::MS_RDONLY,
            SysrootMode::ReadWrite => MsFlags::empty(),
        }
    }
}

pub struct Sysroot {
    repo: Repo,
    deploys_dir: PathBuf,
}

impl Sysroot {
    pub fn new(mode: SysrootMode) -> Result<Self, SysrootError> {
        let sysroot_path = Path::new(ROOT_DIR).join(SYSROOT_DIR);

        if !mounts()?.iter().any(|entry| Path::new(&entry.fs_file) == sysroot_path) {
            return Err(SysrootError::SysrootNotMounted);
        }

        unshare(CloneFlags::CLONE_NEWNS)?;
        mount(
            None::<&str>,
            ROOT_DIR,
            None::<&str>,
            MsFlags::MS_REC | MsFlags::MS_PRIVATE,
            None::<&str>,
        )?;
        mount(
            None::<&str>,
            &sysroot_path,
            None::<&str>,
            MsFlags::MS_REMOUNT | MsFlags::MS_BIND | MsFlags::from(mode),
            None::<&str>,
        )?;

        Self::open(&sysroot_path)
    }

    pub fn init(root: &Path) -> Result<Self, SysrootError> {
        set_permissions(root, Permissions::from_mode(SYSROOT_ROOT_MODE))?;

        let deploys_dir = root.join(DEPLOYS_DIR);
        create_dir_all(&deploys_dir)?;

        let repo = Repo::init(&root.join(REPO_DIR))?;

        Ok(Self { repo, deploys_dir })
    }

    pub fn open(root: &Path) -> Result<Self, SysrootError> {
        let deploys_dir = root.join(DEPLOYS_DIR);
        if !deploys_dir.try_exists()? {
            return Err(SysrootError::DeploysDirNotFound);
        }

        let repo_dir = root.join(REPO_DIR);
        if !repo_dir.try_exists()? {
            return Err(SysrootError::RepoDirNotFound);
        }

        let repo = Repo::open(&repo_dir)?;

        Ok(Self { repo, deploys_dir })
    }

    pub fn repo(&self) -> &Repo {
        &self.repo
    }

    pub fn list(&self) -> Result<impl Iterator<Item = Result<SystemDeploy, SysrootError>>, SysrootError> {
        Ok(read_dir(&self.deploys_dir)?.filter_map(|entry| self.deploy_entry(entry).transpose()))
    }

    pub fn current(&self) -> Result<SystemDeploy, SysrootError> {
        let target = match read_link(self.deploys_dir.join(CURRENT_LINK)) {
            Ok(target) => target,
            Err(error) if error.kind() == IoErrorKind::NotFound => return Err(SysrootError::PointerNotFound),
            Err(error) => return Err(error.into()),
        };

        let digest = target
            .to_str()
            .and_then(|deploy_name| Digest::from_hex(deploy_name).ok())
            .ok_or(SysrootError::InvalidPointer)?;

        Ok(SystemDeploy::open(self, digest)?)
    }

    pub fn set_current(&self, deploy: &SystemDeploy) -> Result<(), SysrootError> {
        let staging_link = self.deploys_dir.join(CURRENT_LINK_STAGING);

        match remove_file(&staging_link) {
            Ok(()) => {}
            Err(error) if error.kind() == IoErrorKind::NotFound => {}
            Err(error) => return Err(error.into()),
        }

        symlink(deploy.digest().to_hex(), &staging_link)?;
        rename(&staging_link, self.deploys_dir.join(CURRENT_LINK))?;

        Ok(())
    }

    fn deploy_entry(&self, entry: Result<DirEntry, IoError>) -> Result<Option<SystemDeploy>, SysrootError> {
        let entry = entry?;
        if !entry.file_type()?.is_dir() {
            return Ok(None);
        }

        let digest = entry
            .file_name()
            .to_str()
            .and_then(|deploy_name| Digest::from_hex(deploy_name).ok())
            .ok_or(SysrootError::InvalidDeploy)?;

        Ok(Some(SystemDeploy::open(self, digest)?))
    }
}
