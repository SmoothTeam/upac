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
use upac_deploy::error::{DeployError, SysrootError};
use upac_deploy::layout::deployment::{DEPLOYS_DIR, REPO_DIR};

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
fn opening_fails_when_the_deploys_dir_is_missing() {
    let scratch = TempDir::new().unwrap();
    create_dir_all(scratch.path().join(REPO_DIR)).unwrap();

    let result = Sysroot::open(scratch.path());

    assert_eq!(result.err(), Some(SysrootError::DeploysDirNotFound));
}

#[test]
fn opening_fails_when_the_repository_dir_is_missing() {
    let scratch = TempDir::new().unwrap();
    create_dir_all(scratch.path().join(DEPLOYS_DIR)).unwrap();

    let result = Sysroot::open(scratch.path());

    assert_eq!(result.err(), Some(SysrootError::RepoDirNotFound));
}

#[test]
fn list_yields_every_deploy_and_skips_the_current_link() {
    let (_scratch, sysroot) = scratch_sysroot();
    let first = SystemDeploy::create(&sysroot, digest(1)).unwrap();
    SystemDeploy::create(&sysroot, digest(2)).unwrap();
    sysroot.set_current(&first).unwrap();

    let mut digests: Vec<Digest> = sysroot
        .list()
        .unwrap()
        .map(|deploy| deploy.unwrap().digest().clone())
        .collect();
    digests.sort_by_key(Digest::to_hex);

    assert_eq!(digests, vec![digest(1), digest(2)]);
}

#[test]
fn the_current_deploy_reads_back_what_was_set() {
    let (_scratch, sysroot) = scratch_sysroot();
    let first = SystemDeploy::create(&sysroot, digest(1)).unwrap();
    let second = SystemDeploy::create(&sysroot, digest(2)).unwrap();

    sysroot.set_current(&first).unwrap();
    sysroot.set_current(&second).unwrap();

    assert_eq!(sysroot.current().unwrap(), second);
}

#[test]
fn the_current_deploy_is_missing_until_it_is_set() {
    let (_scratch, sysroot) = scratch_sysroot();

    assert_eq!(sysroot.current().err(), Some(SysrootError::PointerNotFound));
}

#[test]
fn a_current_link_to_a_removed_deploy_fails() {
    let (_scratch, sysroot) = scratch_sysroot();
    let deploy = SystemDeploy::create(&sysroot, digest(1)).unwrap();
    sysroot.set_current(&deploy).unwrap();

    deploy.remove().unwrap();

    assert_eq!(
        sysroot.current().err(),
        Some(SysrootError::Deploy(DeployError::NotFound))
    );
}
