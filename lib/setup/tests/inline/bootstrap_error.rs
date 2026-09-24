// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::io::{Error as IoError, ErrorKind as IoErrorKind};

use nix::errno::Errno;

use upac_abi::error::ErrorKind;

use super::BootstrapError;

use crate::commands::partition::error::PartitionError;

#[test]
fn io_error_maps_to_setup_error_io_with_the_same_kind() {
    let error = IoError::new(IoErrorKind::PermissionDenied, "denied");

    assert_eq!(
        BootstrapError::from(error),
        BootstrapError::Io(IoErrorKind::PermissionDenied)
    );
}

#[test]
fn errno_maps_to_mount() {
    assert_eq!(BootstrapError::from(Errno::EBUSY), BootstrapError::Mount(Errno::EBUSY));
}

#[test]
fn partition_error_is_wrapped_and_delegates_its_error_kind() {
    let error = BootstrapError::from(PartitionError::InvalidPartitionLayout);

    assert_eq!(error, BootstrapError::Partition(PartitionError::InvalidPartitionLayout));
    assert_eq!(ErrorKind::from(error), ErrorKind::InvalidEntry);
}

#[test]
fn unsupported_deploy_fs_maps_to_invalid_entry() {
    assert_eq!(
        ErrorKind::from(BootstrapError::UnsupportedDeployFs),
        ErrorKind::InvalidEntry
    );
}

#[test]
fn not_esp_partition_maps_to_invalid_entry() {
    assert_eq!(
        ErrorKind::from(BootstrapError::NotEspPartition),
        ErrorKind::InvalidEntry
    );
}
