// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later

use std::fs::create_dir_all;
use std::path::{Path, PathBuf};

use nix::mount::{MsFlags, mount, umount};

use upac::composefs::repository::{self, ObjectID};
use upac::layout::deployment::{DEPLOYS_DIR, NEXT_SEQ_PATH, REPO_DIR};

use composefs::repository::Repository;

use crate::error::SetupError;

pub struct TargetSysroot {
    mount_point: PathBuf,
    deploy_dir: PathBuf,
    repository: Repository<ObjectID>,
}

impl TargetSysroot {
    pub fn new(device_path: &Path, mount_point: PathBuf, filesystem_type: &str) -> Result<Self, SetupError> {
        create_dir_all(&mount_point)?;
        mount(
            Some(device_path),
            &mount_point,
            Some(filesystem_type),
            MsFlags::empty(),
            None::<&str>,
        )?;

        let deploy_dir = mount_point.join(DEPLOYS_DIR);
        create_dir_all(&deploy_dir)?;

        let (repository, _freshly_initialized) = repository::init(&mount_point.join(REPO_DIR))?;

        Ok(Self {
            mount_point,
            deploy_dir,
            repository,
        })
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
}

impl Drop for TargetSysroot {
    fn drop(&mut self) {
        let _ = umount(&self.mount_point);
    }
}
