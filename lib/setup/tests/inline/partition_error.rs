// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::io::{Error as IoError, ErrorKind as IoErrorKind};

use gptman::Error as GptError;
use gptman::linux::BlockError as GptBlockError;

use nix::errno::Errno;

use upac_abi::error::ErrorKind;

use super::PartitionError;

#[test]
fn gpt_error_io_maps_to_partition_error_io_with_the_same_kind() {
    let error = GptError::Io(IoError::new(IoErrorKind::PermissionDenied, "denied"));

    assert_eq!(
        PartitionError::from(error),
        PartitionError::Io(IoErrorKind::PermissionDenied)
    );
}

#[test]
fn gpt_error_invalid_signature_maps_to_table_not_found() {
    assert_eq!(
        PartitionError::from(GptError::InvalidSignature),
        PartitionError::TableNotFound
    );
}

#[test]
fn gpt_error_no_space_left_maps_directly() {
    assert_eq!(PartitionError::from(GptError::NoSpaceLeft), PartitionError::NoSpaceLeft);
}

#[test]
fn gpt_error_invalid_partition_boundaries_maps_to_invalid_partition_layout() {
    assert_eq!(
        PartitionError::from(GptError::InvalidPartitionBoundaries),
        PartitionError::InvalidPartitionLayout
    );
}

#[test]
fn gpt_block_error_not_block_maps_to_not_block_device() {
    assert_eq!(
        PartitionError::from(GptBlockError::NotBlock),
        PartitionError::NotBlockDevice
    );
}

#[test]
fn gpt_block_error_reread_table_keeps_the_errno() {
    assert_eq!(
        PartitionError::from(GptBlockError::RereadTable(Errno::EBUSY)),
        PartitionError::RereadFailed(Errno::EBUSY)
    );
}

#[test]
fn partition_specific_variants_map_to_the_expected_error_kinds() {
    assert_eq!(
        ErrorKind::from(PartitionError::DeviceNotEmpty),
        ErrorKind::AlreadyExists
    );
    assert_eq!(ErrorKind::from(PartitionError::TableNotFound), ErrorKind::NotFound);
    assert_eq!(ErrorKind::from(PartitionError::NoFreeSlot), ErrorKind::NoSpaceLeft);
    assert_eq!(ErrorKind::from(PartitionError::LabelTooLong), ErrorKind::InvalidEntry);
    assert_eq!(ErrorKind::from(PartitionError::InvalidSize), ErrorKind::InvalidEntry);
    assert_eq!(
        ErrorKind::from(PartitionError::PartitionNotReady),
        ErrorKind::NotInitialized
    );
    assert_eq!(
        ErrorKind::from(PartitionError::PartitionKindNotFound),
        ErrorKind::NotFound
    );
    assert_eq!(
        ErrorKind::from(PartitionError::PartitionKindAmbiguous),
        ErrorKind::InvalidEntry
    );
}
