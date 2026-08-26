// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later

use std::fs::create_dir_all;
use std::path::{Path, PathBuf};

use nix::mount::{MsFlags, mount, umount};

use composefs::repository::Repository;

use upac::composefs::repository::{self, ObjectID};
use upac::layout::boot::ESP_MOUNT_PRIMARY;
use upac::layout::deployment::{DEPLOYS_DIR, NEXT_SEQ_PATH, REPO_DIR};

use upac_abi::FsKind;

use upac_types::PartitionMount;

use crate::data::SetupWholeDiskData;
use crate::error::SetupError;
use crate::format::FormatTarget;
use crate::partition::DiskLayout;

pub struct TargetSysroot {
    mount_point: PathBuf,
    deploy_dir: PathBuf,
    repository: Repository<ObjectID>,
    mounted: Vec<PathBuf>,
}

impl TargetSysroot {
    pub fn new(
        deploy_device: &Path, deploy_fs: FsKind, esp_device: &Path, mount_point: PathBuf,
        extra_mounts: &[PartitionMount],
    ) -> Result<Self, SetupError> {
        create_dir_all(&mount_point)?;

        mount(
            Some(deploy_device),
            &mount_point,
            Some(deploy_fs.as_ref()),
            MsFlags::empty(),
            None::<&str>,
        )?;

        let mut mounted = vec![mount_point.clone()];

        let deploy_dir = mount_point.join(DEPLOYS_DIR);
        create_dir_all(&deploy_dir)?;

        let (repository, _freshly_initialized) = repository::init(&mount_point.join(REPO_DIR))?;

        let esp_mount_point = mount_point.join(ESP_MOUNT_PRIMARY.trim_start_matches('/'));

        create_dir_all(&esp_mount_point)?;

        mount(
            Some(esp_device),
            &esp_mount_point,
            Some("vfat"),
            MsFlags::empty(),
            None::<&str>,
        )?;
        mounted.push(esp_mount_point);

        for extra in extra_mounts {
            let target = mount_point.join(extra.mount_path.trim_start_matches('/'));
            create_dir_all(&target)?;
            mount(
                Some(Path::new(&extra.device_path)),
                &target,
                Some(extra.fs_kind.as_ref()),
                MsFlags::empty(),
                None::<&str>,
            )?;
            mounted.push(target);
        }

        Ok(Self {
            mount_point,
            deploy_dir,
            repository,
            mounted,
        })
    }

    pub fn create_whole_disk(data: &SetupWholeDiskData) -> Result<Self, SetupError> {
        let layout = DiskLayout::create(
            Path::new(data.device_path),
            data.esp_size_mib,
            data.deploy_size_mib,
            &data.extra_partitions,
        )?;

        let esp_path = layout.esp_path();
        FormatTarget {
            device_path: &esp_path,
            label: Some("ESP"),
        }
        .format_esp()?;

        let deploy_path = layout.deploy_path();
        FormatTarget {
            device_path: &deploy_path,
            label: Some("upac-deploy"),
        }
        .format(data.deploy_fs, data.node_size, data.sector_size)?;

        let extra_paths = layout.extra_paths();
        let mut extra_mounts = Vec::with_capacity(data.extra_partitions.len());

        for (spec, path) in data.extra_partitions.iter().zip(extra_paths.iter()) {
            FormatTarget {
                device_path: path,
                label: Some(&spec.mount_path),
            }
            .format(spec.fs_kind, 0, 0)?;

            extra_mounts.push(PartitionMount {
                mount_path: spec.mount_path.clone(),
                device_path: path.display().to_string(),
                fs_kind: spec.fs_kind,
            });
        }

        Self::new(
            &deploy_path,
            data.deploy_fs,
            &esp_path,
            PathBuf::from(data.mount_point()),
            &extra_mounts,
        )
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
}

impl Drop for TargetSysroot {
    fn drop(&mut self) {
        for mount_point in self.mounted.iter().rev() {
            let _ = umount(mount_point);
        }
    }
}
