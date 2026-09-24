// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::io::ErrorKind as IoErrorKind;

use upac_abi::error::ErrorKind;

use super::FormatError;

use crate::commands::partition::error::PartitionError;
use crate::wipe::WipeError;

#[test]
fn wipe_error_maps_onto_the_matching_format_error_variant() {
    assert_eq!(
        FormatError::from(WipeError::Io(IoErrorKind::NotFound)),
        FormatError::Io(IoErrorKind::NotFound)
    );
    assert_eq!(
        FormatError::from(WipeError::DeviceNotEmpty),
        FormatError::DeviceNotEmpty
    );
    assert_eq!(FormatError::from(WipeError::WipeFailed), FormatError::WipeFailed);
}

#[test]
fn partition_error_is_wrapped_and_delegates_its_error_kind() {
    let error = FormatError::from(PartitionError::TableNotFound);

    assert_eq!(error, FormatError::Partition(PartitionError::TableNotFound));
    assert_eq!(ErrorKind::from(error), ErrorKind::NotFound);
}

#[test]
fn format_specific_variants_map_to_the_expected_error_kinds() {
    assert_eq!(ErrorKind::from(FormatError::NotEspPartition), ErrorKind::InvalidEntry);
    assert_eq!(
        ErrorKind::from(FormatError::InvalidFormatParams),
        ErrorKind::InvalidEntry
    );
    assert_eq!(ErrorKind::from(FormatError::DeviceNotEmpty), ErrorKind::AlreadyExists);
    assert_eq!(ErrorKind::from(FormatError::MkfsFailed), ErrorKind::WriteFailed);
}
