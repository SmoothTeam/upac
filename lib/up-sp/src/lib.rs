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

use self::layout::bootstrap::SCRATCH_DIR;
use self::probe::detect_filesystem;

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
    esp_mount_point: Option<PathBuf>,
    scratch_dir: PathBuf,
}

impl MountedTarget {
    pub fn mount(deploy_device: &Path, esp_device: Option<&Path>, mount_point: PathBuf) -> Result<Self, ErrorKind> {
        let deploy_fs = match detect_filesystem(deploy_device)? {
            Some(FsKind::Xfs) | None => return Err(ErrorKind::UnsupportedFilesystem),
            Some(fs_kind) => fs_kind,
        };

        create_dir_all(&mount_point)?;
        Self::mount_device(deploy_device, &mount_point, deploy_fs)?;

        let mut mounted = Self {
            scratch_dir: mount_point.join(SCRATCH_DIR),
            root: mount_point,
            esp_mount_point: None,
        };

        if let Some(esp_device) = esp_device {
            let esp_mount_point = mounted.root.join(ESP_MOUNT_PRIMARY.trim_start_matches('/'));
            create_dir_all(&esp_mount_point)?;
            Self::mount_device(esp_device, &esp_mount_point, FsKind::Vfat)?;
            mounted.esp_mount_point = Some(esp_mount_point);
        }

        create_dir_all(&mounted.scratch_dir)?;

        Ok(mounted)
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn esp_mount_point(&self) -> Option<&Path> {
        self.esp_mount_point.as_deref()
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

        if let Some(esp_mount_point) = &self.esp_mount_point {
            Self::unmount(esp_mount_point);
            let _ = remove_dir(esp_mount_point);
        }

        Self::unmount(&self.root);
    }
}
