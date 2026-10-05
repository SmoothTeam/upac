// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::io::{Error as IoError, ErrorKind as IoErrorKind};

use anyhow::anyhow;

use nix::errno::Errno;

use rsmount::errors::MountInfoError;

use upac_composefs::error::RepoError;

use upac_deploy::error::{BootEntryError, EspError, SysrootError};

use upac_types::error::ErrorKind;

#[test]
fn mount_info_error_maps_to_mount_info_unavailable() {
    let error = MountInfoError::Creation("boom".to_owned());

    assert_eq!(SysrootError::from(error), SysrootError::MountInfoUnavailable);
}

#[test]
fn io_error_maps_to_sysroot_dir_unavailable() {
    let error = IoError::new(IoErrorKind::PermissionDenied, "denied");

    assert_eq!(SysrootError::from(error), SysrootError::SysrootDirUnavailable);
}

#[test]
fn errno_maps_to_the_system_variant_with_the_same_errno() {
    assert_eq!(SysrootError::from(Errno::ENOSPC), SysrootError::System(Errno::ENOSPC));
}

#[test]
fn every_variant_maps_to_the_documented_error_kind() {
    let cases = [
        (SysrootError::MountInfoUnavailable, ErrorKind::Unexpected),
        (SysrootError::SysrootNotMounted, ErrorKind::NotFound),
        (SysrootError::SysrootDirUnavailable, ErrorKind::NotFound),
        (SysrootError::DeploysDirNotFound, ErrorKind::NotFound),
        (SysrootError::RepoDirNotFound, ErrorKind::NotFound),
        (SysrootError::Repository(RepoError::NotFound), ErrorKind::NotFound),
        (SysrootError::System(Errno::EIO), ErrorKind::Unexpected),
    ];

    for (error, expected) in cases {
        assert_eq!(ErrorKind::from(error), expected);
    }
}

#[test]
fn every_boot_entry_variant_maps_to_the_documented_error_kind() {
    let cases = [
        (BootEntryError::NoBootResource, ErrorKind::NotFound),
        (BootEntryError::AmbiguousBootResource, ErrorKind::InvalidEntry),
        (BootEntryError::UnsupportedBootResource, ErrorKind::InvalidEntry),
        (BootEntryError::Repository(RepoError::NotFound), ErrorKind::NotFound),
        (BootEntryError::Unexpected, ErrorKind::Unexpected),
    ];

    for (error, expected) in cases {
        assert_eq!(ErrorKind::from(error), expected);
    }
}

#[test]
fn an_anyhow_error_from_composefs_boot_maps_to_unexpected() {
    assert_eq!(BootEntryError::from(anyhow!("boom")), BootEntryError::Unexpected);
}

#[test]
fn esp_errors_map_to_the_documented_error_kind() {
    assert_eq!(ErrorKind::from(EspError::MountInfoUnavailable), ErrorKind::Unexpected);
    assert_eq!(ErrorKind::from(EspError::NotFound), ErrorKind::NotFound);
}
