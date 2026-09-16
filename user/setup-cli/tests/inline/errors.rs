// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: GPL-3.0-only

use upac_abi::error::{CError, ErrorDomain, ErrorKind};

use upac_types::states::SetupStateId;

use crate::locale;

use super::{AbiMismatch, LibError};

fn localized(state: SetupStateId, error: ErrorKind) -> String {
    locale::init_for_test();

    let error = CError {
        domain: ErrorDomain::Setup,
        state: state as u32,
        error,
    };

    LibError { error }.to_string()
}

#[test]
fn prefixes_the_message_with_the_localized_failing_stage_name() {
    let message = localized(SetupStateId::ImportPackage, ErrorKind::Unexpected);

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
        let message = localized(SetupStateId::Setup, kind);

        assert_eq!(message, format!("Setup: {expected}"));
    }
}

#[test]
fn abi_mismatch_embeds_both_versions() {
    locale::init_for_test();

    let message = AbiMismatch { got: 1, expected: 2 }.to_string();

    assert_eq!(message, "ABI version mismatch (1 → 2)");
}
