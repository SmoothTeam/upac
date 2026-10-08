// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::fs::{File, create_dir_all, write};

use composefs::generic_tree::Stat;
use composefs::repository::{Repository, RepositoryConfig};

use nix::fcntl::AT_FDCWD;

use tempfile::{Builder, TempDir};

use upac_composefs::tree::Tree;
use upac_composefs::{Digest, ObjectID};

use upac_types::booter::BootResourceKind;

use upac_deploy::Sysroot;
use upac_deploy::boot::WrittenBootEntry;
use upac_deploy::error::BootEntryError;
use upac_deploy::layout::deployment::{DEPLOYS_DIR, REPO_DIR};

fn scratch_dir(name: &str) -> TempDir {
    Builder::new().prefix(name).tempdir().unwrap()
}

fn scratch_sysroot() -> (TempDir, Sysroot) {
    let scratch = scratch_dir("boot-sysroot");

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

fn source_file(dir_name: &str, content: &[u8]) -> File {
    let dir = scratch_dir(dir_name);
    let path = dir.path().join("source");
    write(&path, content).unwrap();

    File::open(&path).unwrap()
}

fn insert_kernel(tree: &mut Tree, kernel_version: &str) {
    if !tree.contains("lib/modules") {
        tree.insert_dir("lib", Stat::uninitialized()).unwrap();
        tree.insert_dir("lib/modules", Stat::uninitialized()).unwrap();
    }

    tree.insert_dir(format!("lib/modules/{kernel_version}"), Stat::uninitialized())
        .unwrap();
    tree.insert_file(
        format!("lib/modules/{kernel_version}/vmlinuz"),
        &source_file(&format!("kernel-{kernel_version}"), b"kernel"),
        Stat::uninitialized(),
    )
    .unwrap();
}

fn prefix_with_kernels(sysroot: &Sysroot, kernel_versions: &[&str]) -> Digest {
    let mut tree = sysroot.repo().empty_tree();
    for kernel_version in kernel_versions {
        insert_kernel(&mut tree, kernel_version);
    }

    tree.commit().unwrap()
}

#[test]
fn a_prefix_without_a_boot_resource_is_rejected() {
    let (_scratch, sysroot) = scratch_sysroot();
    let prefix_digest = prefix_with_kernels(&sysroot, &[]);
    let esp = scratch_dir("boot-none-esp");

    let result = sysroot.write_boot_entry(&prefix_digest, esp.path(), BootResourceKind::Bls);

    assert_eq!(result, Err(BootEntryError::NoBootResource));
}

#[test]
fn a_prefix_with_two_kernels_is_ambiguous() {
    let (_scratch, sysroot) = scratch_sysroot();
    let prefix_digest = prefix_with_kernels(&sysroot, &["6.6.0", "6.7.0"]);
    let esp = scratch_dir("boot-ambiguous-esp");

    let result = sysroot.write_boot_entry(&prefix_digest, esp.path(), BootResourceKind::Bls);

    assert_eq!(result, Err(BootEntryError::AmbiguousBootResource));
}

#[test]
fn a_uki_cannot_be_built_from_a_kernel_without_an_initramfs() {
    let (_scratch, sysroot) = scratch_sysroot();
    let prefix_digest = prefix_with_kernels(&sysroot, &["6.6.0"]);
    let esp = scratch_dir("boot-uki-esp");

    let result = sysroot.write_boot_entry(&prefix_digest, esp.path(), BootResourceKind::Uki);

    assert_eq!(result, Err(BootEntryError::InitramfsMissing));
}

#[test]
fn a_bls_entry_is_named_after_the_prefix() {
    let entry = WrittenBootEntry::Bls("deadbeef".to_owned());

    assert_eq!(entry.entry_name(), "deadbeef");
    assert_eq!(entry.into_entry_name(), "deadbeef");
}
