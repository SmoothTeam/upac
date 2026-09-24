// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: GPL-3.0-only

use upac_abi::error::{ErrorDomain, ErrorKind};

use upac_types::error::Error as AbiError;
use upac_types::state::setup::{BootstrapStateId, FormatStateId, PartitionAddStateId, PartitionTableStateId};

use crate::locale;

use super::{AbiMismatch, LibError};

fn localized(state: BootstrapStateId, kind: ErrorKind) -> String {
    locale::init_for_test();

    let error = AbiError {
        domain: ErrorDomain::Bootstrap,
        state: state as u32,
        kind,
    };

    LibError(error).to_string()
}

#[test]
fn prefixes_the_message_with_the_localized_failing_stage_name() {
    let message = localized(BootstrapStateId::ImportPackage, ErrorKind::Unexpected);

    assert_eq!(message, "Importing package: Unexpected error");
}

#[test]
fn every_error_kind_has_its_own_localized_message() {
    let cases = [
        (ErrorKind::Unexpected, "Unexpected error"),
        (ErrorKind::OutOfMemory, "Out of memory"),
        (ErrorKind::NotFound, "File not found"),
        (ErrorKind::AlreadyExists, "Already exists"),
        (ErrorKind::PermissionDenied, "Permission denied"),
        (ErrorKind::InvalidPath, "Invalid path"),
        (ErrorKind::NoSpaceLeft, "No space left"),
        (ErrorKind::Cancelled, "Cancelled"),
        (ErrorKind::ReadFailed, "Read failed"),
        (ErrorKind::WriteFailed, "Write failed"),
        (ErrorKind::NotInitialized, "Not initialized"),
        (ErrorKind::AbiMismatch, "ABI mismatch"),
        (ErrorKind::InvalidEntry, "Invalid entry"),
    ];

    for (kind, expected) in cases {
        let message = localized(BootstrapStateId::Setup, kind);

        assert_eq!(message, format!("Setup: {expected}"));
    }
}

#[test]
fn resolves_the_failing_stage_through_the_partition_domains() {
    locale::init_for_test();

    let table_error = AbiError {
        domain: ErrorDomain::PartitionTable,
        state: PartitionTableStateId::WriteTable as u32,
        kind: ErrorKind::WriteFailed,
    };
    let add_error = AbiError {
        domain: ErrorDomain::PartitionAdd,
        state: PartitionAddStateId::Settle as u32,
        kind: ErrorKind::NotInitialized,
    };

    assert_eq!(
        LibError(table_error).to_string(),
        "Writing partition table: Write failed"
    );
    assert_eq!(
        LibError(add_error).to_string(),
        "Waiting for partition device: Not initialized"
    );
}

#[test]
fn resolves_the_failing_stage_through_the_format_domain() {
    locale::init_for_test();

    let error = AbiError {
        domain: ErrorDomain::Format,
        state: FormatStateId::Verify as u32,
        kind: ErrorKind::InvalidEntry,
    };

    assert_eq!(LibError(error).to_string(), "Verifying ESP partition: Invalid entry");
}

#[test]
fn abi_mismatch_embeds_both_versions() {
    locale::init_for_test();

    let message = AbiMismatch { got: 1, expected: 2 }.to_string();

    assert_eq!(message, "ABI version mismatch (1 → 2)");
}
