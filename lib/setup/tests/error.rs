// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::io::{Error as IoError, ErrorKind as IoErrorKind};

use gptman::Error as GptError;
use gptman::linux::BlockError as GptBlockError;

use nix::errno::Errno;

use upac_setup::error::SetupError;

#[test]
fn gpt_error_io_maps_to_setup_error_io_with_the_same_kind() {
    let error = GptError::Io(IoError::new(IoErrorKind::PermissionDenied, "denied"));

    assert_eq!(SetupError::from(error), SetupError::Io(IoErrorKind::PermissionDenied));
}

#[test]
fn gpt_error_no_space_left_maps_directly() {
    assert_eq!(SetupError::from(GptError::NoSpaceLeft), SetupError::NoSpaceLeft);
}

#[test]
fn gpt_error_invalid_partition_boundaries_maps_to_invalid_partition_layout() {
    assert_eq!(
        SetupError::from(GptError::InvalidPartitionBoundaries),
        SetupError::InvalidPartitionLayout
    );
}

#[test]
fn gpt_error_falls_back_to_unexpected_for_other_variants() {
    assert_eq!(SetupError::from(GptError::InvalidSignature), SetupError::Unexpected);
}

#[test]
fn gpt_block_error_metadata_maps_to_setup_error_io_with_the_same_kind() {
    let error = GptBlockError::Metadata(IoError::new(IoErrorKind::NotFound, "missing"));

    assert_eq!(SetupError::from(error), SetupError::Io(IoErrorKind::NotFound));
}

#[test]
fn gpt_block_error_not_block_maps_directly() {
    assert_eq!(SetupError::from(GptBlockError::NotBlock), SetupError::NotBlockDevice);
}

#[test]
fn gpt_block_error_reread_table_maps_to_reread_failed_with_the_same_errno() {
    let error = GptBlockError::RereadTable(Errno::ENOSPC);

    assert_eq!(SetupError::from(error), SetupError::RereadFailed(Errno::ENOSPC));
}
