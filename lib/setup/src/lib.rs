// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::fs::{create_dir_all, remove_dir, remove_dir_all};
use std::io::Error as IoError;
use std::path::{Path, PathBuf};

use nix::mount::{MntFlags, MsFlags, mount, umount, umount2};

use upac_types::error::ErrorKind;
use upac_types::request::format::FsKind;

use upac_deploy::layout::boot::ESP_MOUNT_PRIMARY;

use self::layout::genesis::SCRATCH_DIR;

pub mod export;

mod archive;
mod commands;
mod gpt;
mod layout {
    include!(concat!(env!("OUT_DIR"), "/layout.rs"));
}
mod probe;
mod wipe;

pub(crate) struct MountedTarget {
    root: PathBuf,
    esp_mount_point: PathBuf,
    esp_device: PathBuf,
    scratch_dir: PathBuf,
}

impl MountedTarget {
    pub fn mount(
        deploy_device: &Path, deploy_fs: FsKind, esp_device: &Path, mount_point: PathBuf,
    ) -> Result<Self, ErrorKind> {
        create_dir_all(&mount_point)?;
        Self::mount_device(deploy_device, &mount_point, deploy_fs)?;

        let esp_mount_point = mount_point.join(ESP_MOUNT_PRIMARY.trim_start_matches('/'));
        let mounted_esp = create_dir_all(&esp_mount_point)
            .map_err(ErrorKind::from)
            .and_then(|()| Self::mount_device(esp_device, &esp_mount_point, FsKind::Vfat));
        if let Err(error) = mounted_esp {
            Self::unmount(&mount_point);
            return Err(error);
        }

        let mounted = Self {
            scratch_dir: mount_point.join(SCRATCH_DIR),
            root: mount_point,
            esp_mount_point,
            esp_device: esp_device.to_owned(),
        };
        create_dir_all(&mounted.scratch_dir)?;

        Ok(mounted)
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn esp_mount_point(&self) -> &Path {
        &self.esp_mount_point
    }

    pub fn esp_device(&self) -> &Path {
        &self.esp_device
    }

    pub fn scratch_dir(&self) -> &Path {
        &self.scratch_dir
    }

    fn mount_device(device: &Path, mount_point: &Path, fs_kind: FsKind) -> Result<(), ErrorKind> {
        mount(
            Some(device),
            mount_point,
            Some(fs_kind.as_ref()),
            MsFlags::empty(),
            None::<&str>,
        )
        .map_err(|errno| IoError::from(errno).into())
    }

    fn unmount(mount_point: &Path) {
        if umount(mount_point).is_err() {
            let _ = umount2(mount_point, MntFlags::MNT_DETACH);
        }
    }
}

impl Drop for MountedTarget {
    fn drop(&mut self) {
        let _ = remove_dir_all(&self.scratch_dir);

        Self::unmount(&self.esp_mount_point);
        let _ = remove_dir(&self.esp_mount_point);

        Self::unmount(&self.root);
    }
}
