// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

#![cfg(feature = "gpt")]

use std::io::{Error as IoError, ErrorKind as IoErrorKind};

use gptman::Error as GptError;
use gptman::linux::BlockError as GptBlockError;

use nix::errno::Errno;

use upac_types::error::ErrorKind;

#[test]
fn a_gpt_io_error_maps_like_any_other_io_error() {
    let error = GptError::Io(IoError::new(IoErrorKind::PermissionDenied, "denied"));

    assert_eq!(ErrorKind::from(error), ErrorKind::PermissionDenied);
}

#[test]
fn a_missing_gpt_signature_means_there_is_no_table() {
    assert_eq!(ErrorKind::from(GptError::InvalidSignature), ErrorKind::NotFound);
}

#[test]
fn a_full_gpt_maps_to_no_space_left() {
    assert_eq!(ErrorKind::from(GptError::NoSpaceLeft), ErrorKind::NoSpaceLeft);
}

#[test]
fn invalid_partition_boundaries_are_an_invalid_entry() {
    assert_eq!(
        ErrorKind::from(GptError::InvalidPartitionBoundaries),
        ErrorKind::InvalidEntry
    );
}

#[test]
fn a_device_that_is_not_a_block_device_is_an_invalid_entry() {
    assert_eq!(ErrorKind::from(GptBlockError::NotBlock), ErrorKind::InvalidEntry);
}

#[test]
fn a_failed_table_reread_is_a_read_failure() {
    assert_eq!(
        ErrorKind::from(GptBlockError::RereadTable(Errno::EBUSY)),
        ErrorKind::ReadFailed
    );
}
