// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::fs::{Permissions, create_dir, create_dir_all, read_dir, read_to_string, remove_dir_all, set_permissions};
use std::io::{Error as IoError, ErrorKind as IoErrorKind};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use composefs::fs::read_file;

use composefs_boot::bootloader::{BootEntry, get_boot_resources};
use composefs_boot::cmdline::ComposefsCmdline;
use composefs_boot::write_boot::write_boot_simple;

use nix::mount::{MsFlags, mount};
use nix::sched::{CloneFlags, unshare};

use procfs::mounts;

use upac_types::booter::BootResourceKind;

use upac_composefs::error::RepoError;
use upac_composefs::fs::WrittenFile;
use upac_composefs::tree::Tree;
use upac_composefs::{Digest, Repo};

use upac_database::MemoryDatabase;
use upac_database::layout::database::DATABASE_PATH;
use upac_database::transaction::TransactionStore;

use self::boot::{WrittenBootEntry, wrap_under_usr};
use self::deployment::Deployment;
use self::deployment::PrefixDeploy;
use self::deployment::meta::PrefixPointer;
use self::error::{BootEntryError, PrefixCreateError, PrefixEditError, PrefixMetaError, PrefixReadError, SysrootError};
use self::etc::SetAsideEtc;
use self::layout::boot::{EFI_LINUX_DIR, KERNEL_ARGUMENTS, UPAC_UKI_TO_SLOT};
use self::layout::deployment::{
    CONFIG_DIR_NAME, DEPLOYS_DIR, DISCARDED_ETC_UPPER_DIR_NAME, LIVE_ETC_UPPER_DIR_NAME, NEXT_PREFIX_FILENAME,
    REPO_DIR, ROOT_DIR, RUNNING_PREFIX_PATH, SYSROOT_DIR, SYSROOT_ROOT_MODE,
};
use self::layout::prefix::DEFAULTS_DIR;
use self::uki::UkiParts;
use self::working::WorkingPrefix;

pub mod boot;
pub mod deployment;
pub mod error;
pub mod etc;
pub mod layout {
    include!(concat!(env!("OUT_DIR"), "/layout.rs"));
}
mod uki;
pub mod working;

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

    pub fn prefix_database(&self, prefix_digest: &Digest) -> Result<MemoryDatabase, PrefixReadError> {
        let tree = self.repo.open_tree(prefix_digest)?;

        Ok(MemoryDatabase::open_in_memory(tree.read_file(DATABASE_PATH)?)?)
    }

    pub fn prefix_defaults(&self, prefix_digest: &Digest) -> Result<Tree, RepoError> {
        let tree = self.repo.open_tree(prefix_digest)?;

        if tree.contains(DEFAULTS_DIR) {
            tree.copy_tree(DEFAULTS_DIR)
        } else {
            Ok(self.repo.empty_tree())
        }
    }

    pub fn working_prefix(&self, base: &PrefixDeploy) -> Result<WorkingPrefix, PrefixReadError> {
        Ok(WorkingPrefix::new(
            self.repo.clone(),
            self.repo.open_tree(base.digest())?,
            self.prefix_database(base.digest())?,
            Some(base.transaction().uuid.to_string()),
        ))
    }

    pub fn empty_prefix(&self) -> Result<WorkingPrefix, PrefixEditError> {
        Ok(WorkingPrefix::new(
            self.repo.clone(),
            self.repo.empty_tree(),
            MemoryDatabase::new_in_memory()?,
            None,
        ))
    }

    pub fn prefix(&self, prefix_digest: &Digest) -> Result<PrefixDeploy, PrefixReadError> {
        let transaction = self
            .prefix_database(prefix_digest)?
            .get_transaction()?
            .ok_or(PrefixReadError::TransactionMissing)?;

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

    pub fn is_live_etc_modified(&self, prefix_digest: &Digest) -> Result<bool, IoError> {
        match read_dir(self.live_etc_upper_dir(prefix_digest)) {
            Ok(mut entries) => Ok(entries.next().is_some()),
            Err(error) if error.kind() == IoErrorKind::NotFound => Ok(false),
            Err(error) => Err(error),
        }
    }

    pub fn set_aside_live_etc(&self, prefix_digest: &Digest) -> Result<SetAsideEtc, IoError> {
        let config_dir = self.deploys_dir.join(prefix_digest.to_hex()).join(CONFIG_DIR_NAME);

        SetAsideEtc::new(
            config_dir.join(LIVE_ETC_UPPER_DIR_NAME),
            config_dir.join(DISCARDED_ETC_UPPER_DIR_NAME),
        )
    }

    pub fn write_boot_entry(
        &self, prefix_digest: &Digest, esp_dir: &Path, wanted: BootResourceKind,
    ) -> Result<WrittenBootEntry, BootEntryError> {
        let tree = self.repo.open_tree(prefix_digest)?;
        let entries = get_boot_resources(&wrap_under_usr(tree.upstream()), self.repo.upstream())?;

        if entries.is_empty() {
            return Err(BootEntryError::NoBootResource);
        }

        let mut kernels: Vec<_> = entries
            .into_iter()
            .filter(|entry| matches!(entry, BootEntry::Type1(_) | BootEntry::UsrLibModulesVmLinuz(_)))
            .collect();

        if kernels.len() > 1 {
            return Err(BootEntryError::AmbiguousBootResource);
        }
        let kernel = kernels.pop().ok_or(BootEntryError::UnsupportedBootResource)?;

        let composefs_argument = ComposefsCmdline::new_v2(prefix_digest.object_id().clone(), false);

        match wanted {
            BootResourceKind::Bls => {
                let written = WrittenBootEntry::Bls(prefix_digest.to_hex());
                let extra_arguments: Vec<&str> = KERNEL_ARGUMENTS.split_whitespace().collect();

                write_boot_simple(
                    self.repo.upstream(),
                    kernel,
                    &composefs_argument,
                    esp_dir,
                    None,
                    Some(written.entry_name()),
                    &extra_arguments,
                )?;

                Ok(written)
            }
            BootResourceKind::Uki => {
                let BootEntry::UsrLibModulesVmLinuz(kernel) = kernel else {
                    return Err(BootEntryError::UnsupportedBootResource);
                };
                let initramfs = kernel.initramfs.as_ref().ok_or(BootEntryError::InitramfsMissing)?;
                let os_release = match &kernel.os_release {
                    Some(os_release) => Some(read_file(os_release, self.repo.upstream())?),
                    None => None,
                };

                UkiParts {
                    kernel: &read_file(&kernel.vmlinuz, self.repo.upstream())?,
                    initramfs: &read_file(initramfs, self.repo.upstream())?,
                    os_release: os_release.as_deref(),
                    cmdline: format!("{} {KERNEL_ARGUMENTS}", composefs_argument.to_cmdline_arg()),
                }
                .build(&esp_dir.join(EFI_LINUX_DIR).join(format!("{UPAC_UKI_TO_SLOT}.efi")))?;

                Ok(WrittenBootEntry::Uki(UPAC_UKI_TO_SLOT.to_owned()))
            }
        }
    }
}
