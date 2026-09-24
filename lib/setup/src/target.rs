// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::fs::{create_dir_all, remove_dir};
use std::mem::ManuallyDrop;
use std::path::{Path, PathBuf};

use nix::mount::{MsFlags, mount, umount};

use composefs::repository::Repository;

use upac::composefs::repository::{self, ObjectID};
use upac::layout::boot::ESP_MOUNT_PRIMARY;
use upac::layout::deployment::{DEPLOYS_DIR, NEXT_SEQ_PATH, REPO_DIR};

use upac_abi::FsKind;

use crate::commands::bootstrap::error::BootstrapError;

pub struct TargetSysroot {
    mount_point: PathBuf,
    deploy_dir: PathBuf,
    esp_device: PathBuf,
    repository: ManuallyDrop<Repository<ObjectID>>,
    mounted: Vec<PathBuf>,
}

impl TargetSysroot {
    pub fn new(
        deploy_device: &Path, deploy_fs: FsKind, esp_device: &Path, mount_point: PathBuf,
    ) -> Result<Self, BootstrapError> {
        create_dir_all(&mount_point)?;

        mount(
            Some(deploy_device),
            &mount_point,
            Some(deploy_fs.as_ref()),
            MsFlags::empty(),
            None::<&str>,
        )?;

        let (deploy_dir, repository, esp_mount_point) = Self::prepare_mounted_deploy(&mount_point, esp_device)
            .inspect_err(|_| {
                let _ = umount(&mount_point);
            })?;

        Ok(Self {
            mounted: vec![mount_point.clone(), esp_mount_point],
            mount_point,
            deploy_dir,
            esp_device: esp_device.to_owned(),
            repository: ManuallyDrop::new(repository),
        })
    }

    fn prepare_mounted_deploy(
        mount_point: &Path, esp_device: &Path,
    ) -> Result<(PathBuf, Repository<ObjectID>, PathBuf), BootstrapError> {
        let deploy_dir = mount_point.join(DEPLOYS_DIR);
        create_dir_all(&deploy_dir)?;

        let (repository, _freshly_initialized) = repository::init(&mount_point.join(REPO_DIR))?;

        let esp_mount_point = mount_point.join(ESP_MOUNT_PRIMARY.trim_start_matches('/'));
        create_dir_all(&esp_mount_point)?;

        mount(
            Some(esp_device),
            &esp_mount_point,
            Some(FsKind::Vfat.as_ref()),
            MsFlags::empty(),
            None::<&str>,
        )?;

        Ok((deploy_dir, repository, esp_mount_point))
    }

    pub fn repository(&self) -> &Repository<ObjectID> {
        &self.repository
    }

    pub fn deploy_dir(&self, prefix_digest: &str) -> PathBuf {
        self.deploy_dir.join(prefix_digest)
    }

    pub fn next_seq_path(&self) -> PathBuf {
        self.mount_point.join(NEXT_SEQ_PATH)
    }

    pub fn esp_mount_point(&self) -> PathBuf {
        self.mount_point.join(ESP_MOUNT_PRIMARY.trim_start_matches('/'))
    }

    pub fn esp_device(&self) -> &Path {
        &self.esp_device
    }
}

impl Drop for TargetSysroot {
    fn drop(&mut self) {
        // SAFETY: `self` is being dropped and `repository` is never accessed again.
        unsafe { ManuallyDrop::drop(&mut self.repository) };

        let Some((base, nested)) = self.mounted.split_first() else {
            return;
        };

        for mount_point in nested.iter().rev() {
            let _ = umount(mount_point);
            let _ = remove_dir(mount_point);
        }

        let _ = umount(base);
    }
}

#[cfg(test)]
impl TargetSysroot {
    pub(crate) fn for_testing(mount_point: PathBuf) -> Result<Self, BootstrapError> {
        create_dir_all(&mount_point)?;

        let deploy_dir = mount_point.join(DEPLOYS_DIR);
        create_dir_all(&deploy_dir)?;

        let (repository, _freshly_initialized) = repository::init_insecure(&mount_point.join(REPO_DIR))?;

        Ok(Self {
            mount_point,
            deploy_dir,
            esp_device: PathBuf::new(),
            repository: ManuallyDrop::new(repository),
            mounted: Vec::new(),
        })
    }
}
