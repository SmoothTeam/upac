// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::fs::create_dir_all;

use composefs::repository::{Repository, RepositoryConfig};

use nix::fcntl::AT_FDCWD;

use tempfile::TempDir;

use upac_composefs::{Digest, ObjectID};

use upac_deploy::Sysroot;
use upac_deploy::deployment::SystemDeploy;
use upac_deploy::error::DeployError;
use upac_deploy::layout::deployment::{DEPLOYS_DIR, REPO_DIR};
use upac_deploy::snapshot::ConfigSnapshot;

fn scratch_sysroot() -> (TempDir, Sysroot) {
    let scratch = TempDir::new().unwrap();

    create_dir_all(scratch.path().join(DEPLOYS_DIR)).unwrap();
    Repository::<ObjectID>::init_path(
        AT_FDCWD,
        scratch.path().join(REPO_DIR),
        RepositoryConfig::default().set_insecure(),
    )
    .unwrap();

    let sysroot = Sysroot::open(scratch.path()).unwrap();

    (scratch, sysroot)
}

fn digest(seed: u8) -> Digest {
    Digest::from_hex(&format!("{seed:02x}").repeat(32)).unwrap()
}

#[test]
fn a_created_deploy_has_its_upper_and_work_dirs() {
    let (_scratch, sysroot) = scratch_sysroot();

    let deploy = SystemDeploy::create(&sysroot, digest(1)).unwrap();

    assert!(deploy.upper_dir().is_dir());
    assert!(deploy.work_dir().is_dir());
    assert_eq!(deploy.digest(), &digest(1));
}

#[test]
fn creating_an_existing_deploy_fails() {
    let (_scratch, sysroot) = scratch_sysroot();
    SystemDeploy::create(&sysroot, digest(1)).unwrap();

    assert_eq!(
        SystemDeploy::create(&sysroot, digest(1)).err(),
        Some(DeployError::AlreadyExists)
    );
}

#[test]
fn a_created_deploy_opens_again() {
    let (_scratch, sysroot) = scratch_sysroot();
    let created = SystemDeploy::create(&sysroot, digest(1)).unwrap();

    assert_eq!(SystemDeploy::open(&sysroot, digest(1)).unwrap(), created);
}

#[test]
fn opening_a_missing_deploy_fails() {
    let (_scratch, sysroot) = scratch_sysroot();

    assert_eq!(
        SystemDeploy::open(&sysroot, digest(1)).err(),
        Some(DeployError::NotFound)
    );
}

#[test]
fn pinning_is_kept_and_can_be_undone() {
    let (_scratch, sysroot) = scratch_sysroot();
    let deploy = SystemDeploy::create(&sysroot, digest(1)).unwrap();

    assert!(!deploy.is_pinned().unwrap());
    deploy.set_pinned(true).unwrap();
    assert!(SystemDeploy::open(&sysroot, digest(1)).unwrap().is_pinned().unwrap());
    deploy.set_pinned(false).unwrap();
    deploy.set_pinned(false).unwrap();
    assert!(!deploy.is_pinned().unwrap());
}

#[test]
fn a_removed_deploy_is_gone() {
    let (_scratch, sysroot) = scratch_sysroot();
    let deploy = SystemDeploy::create(&sysroot, digest(1)).unwrap();

    deploy.remove().unwrap();

    assert_eq!(
        SystemDeploy::open(&sysroot, digest(1)).err(),
        Some(DeployError::NotFound)
    );
}

#[test]
fn a_new_deploy_has_no_snapshots() {
    let (_scratch, sysroot) = scratch_sysroot();
    let deploy = SystemDeploy::create(&sysroot, digest(1)).unwrap();

    assert!(deploy.snapshots().unwrap().is_empty());
}

#[test]
fn added_snapshots_read_back_in_time_order() {
    let (_scratch, sysroot) = scratch_sysroot();
    let deploy = SystemDeploy::create(&sysroot, digest(1)).unwrap();
    let before = ConfigSnapshot::new(digest(2), Some("before rollback".to_owned()));
    let after = ConfigSnapshot::new(digest(3), None);

    deploy.add_snapshot(&after).unwrap();
    deploy.add_snapshot(&before).unwrap();

    assert_eq!(deploy.snapshots().unwrap(), vec![before, after]);
}

#[test]
fn the_same_snapshot_cannot_be_added_twice() {
    let (_scratch, sysroot) = scratch_sysroot();
    let deploy = SystemDeploy::create(&sysroot, digest(1)).unwrap();
    let snapshot = ConfigSnapshot::new(digest(2), None);

    deploy.add_snapshot(&snapshot).unwrap();

    assert!(deploy.add_snapshot(&snapshot).is_err());
    assert_eq!(deploy.snapshots().unwrap().len(), 1);
}
