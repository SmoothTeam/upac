// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::fs::{create_dir, read_dir, read_to_string, remove_dir_all};
use std::io::ErrorKind as IoErrorKind;
use std::path::{Path, PathBuf};

use nix::mount::{MsFlags, mount};
use nix::sched::{CloneFlags, unshare};

use rsmount::tables::MountInfo;

use upac_composefs::error::RepoError;
use upac_composefs::fs::WrittenFile;
use upac_composefs::{Digest, Repo};

use upac_database::MemoryDatabase;
use upac_database::layout::database::DATABASE_PATH;
use upac_database::transaction::TransactionStore;

use self::deployment::Deployment;
use self::deployment::PrefixDeploy;
use self::deployment::meta::PrefixPointer;
use self::error::{PrefixCreateError, PrefixMetaError, PrefixReadError, SysrootError};
use self::layout::deployment::{
    CONFIG_DIR_NAME, DEPLOYS_DIR, LIVE_ETC_UPPER_DIR_NAME, NEXT_PREFIX_FILENAME, REPO_DIR, ROOT_DIR,
    RUNNING_PREFIX_PATH, SYSROOT_DIR,
};

pub mod deployment;
pub mod error;
pub mod layout {
    include!(concat!(env!("OUT_DIR"), "/layout.rs"));
}

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

        let mut mount_table = MountInfo::new()?;
        mount_table.import_mountinfo()?;
        if mount_table.find_target(&sysroot_path).is_none() {
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

    pub fn prefix(&self, prefix_digest: &Digest) -> Result<PrefixDeploy, PrefixReadError> {
        let tree = self.repo.open_tree(prefix_digest)?;

        let database_bytes = tree.read_file(DATABASE_PATH)?;
        let database = MemoryDatabase::open_in_memory(database_bytes)?;
        let transaction = database.get_transaction()?.ok_or(PrefixReadError::TransactionMissing)?;

        Ok(PrefixDeploy::read(
            prefix_digest.clone(),
            transaction,
            &self.deploys_dir.join(prefix_digest.to_hex()),
        )?)
    }

    pub fn prefixes(&self) -> Result<Vec<PrefixDeploy>, PrefixReadError> {
        let mut prefixes = Vec::new();

        for entry in read_dir(&self.deploys_dir)? {
            let entry = entry?;

            if !entry.file_type()?.is_dir() {
                continue;
            }

            if let Some(prefix_hex) = entry.file_name().to_str() {
                prefixes.push(self.prefix(&Digest::from_hex(prefix_hex)?)?);
            }
        }

        Ok(prefixes)
    }

    pub fn running_prefix(&self) -> Result<PrefixDeploy, PrefixReadError> {
        let running_digest = read_to_string(RUNNING_PREFIX_PATH).map_err(PrefixMetaError::from)?;

        self.prefix(&Digest::from_hex(running_digest.trim())?)
    }

    pub fn next_prefix(&self) -> Result<PrefixDeploy, PrefixReadError> {
        let pointer = PrefixPointer::read(&self.deploys_dir.join(NEXT_PREFIX_FILENAME))?;

        self.prefix(&pointer.prefix_digest)
    }

    pub fn set_next_prefix(&self, prefix: &PrefixDeploy) -> Result<WrittenFile, PrefixMetaError> {
        let pointer = PrefixPointer {
            prefix_digest: prefix.digest().clone(),
        };

        pointer.write(&self.deploys_dir.join(NEXT_PREFIX_FILENAME))
    }

    pub fn create_prefix(&self, prefix: &PrefixDeploy) -> Result<(), PrefixCreateError> {
        let prefix_dir = self.deploys_dir.join(prefix.digest().to_hex());

        if prefix_dir.try_exists()? {
            return Err(PrefixCreateError::AlreadyExists(prefix.digest().clone()));
        }

        create_dir(&prefix_dir)?;

        let populated = create_dir(prefix_dir.join(CONFIG_DIR_NAME))
            .map_err(PrefixCreateError::from)
            .and_then(|()| prefix.write(&prefix_dir).map_err(PrefixCreateError::from));

        if let Err(error) = populated {
            let _ = self.remove_prefix(prefix.digest());
            return Err(error);
        }

        Ok(())
    }

    pub fn save_prefix(&self, prefix: &PrefixDeploy) -> Result<WrittenFile, PrefixMetaError> {
        prefix.write(&self.deploys_dir.join(prefix.digest().to_hex()))
    }

    pub fn remove_prefix(&self, prefix_digest: &Digest) -> Result<(), RepoError> {
        match remove_dir_all(self.deploys_dir.join(prefix_digest.to_hex())) {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == IoErrorKind::NotFound => Ok(()),
            Err(error) => Err(error.into()),
        }
    }

    pub fn live_etc_upper_dir(&self, prefix_digest: &Digest) -> PathBuf {
        self.deploys_dir
            .join(prefix_digest.to_hex())
            .join(CONFIG_DIR_NAME)
            .join(LIVE_ETC_UPPER_DIR_NAME)
    }
}
