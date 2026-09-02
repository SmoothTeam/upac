// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: GPL-3.0-only

use upac_abi::error::{CError, ErrorDomain, ErrorKind};

use crate::locale;
use crate::types::errors::{AbiMismatch, LibError, StageName};

#[test]
fn abi_mismatch_display_includes_both_versions() {
    locale::init_for_test();

    let message = AbiMismatch { got: 1, expected: 2 }.to_string();

    assert_eq!(message, "ABI version mismatch (1 → 2)");
}

#[test]
fn stage_name_resolves_the_localized_first_stage_of_every_domain() {
    locale::init_for_test();

    let cases = [
        (ErrorDomain::Uninstall, "Pre-hooks"),
        (ErrorDomain::Install, "Pre-hooks"),
        (ErrorDomain::Rollback, "Pre-hooks"),
        (ErrorDomain::Commit, "Pre-hooks"),
        (ErrorDomain::Files, "Pre-hooks"),
        (ErrorDomain::Update, "Pre-hooks"),
        (ErrorDomain::Gc, "Pruning"),
        (ErrorDomain::Pin, "Set pinned"),
        (ErrorDomain::Mime, "Preparing"),
        (ErrorDomain::ListPackages, "Fetching"),
        (ErrorDomain::ListConfig, "Fetching"),
        (ErrorDomain::ListPrefix, "Fetching"),
        (ErrorDomain::ListHistory, "Fetching"),
        (ErrorDomain::DiffPrefix, "Preparing"),
        (ErrorDomain::DiffConfig, "Preparing"),
        (ErrorDomain::DiffPackages, "Preparing"),
        (ErrorDomain::Diff, "Preparing"),
        (ErrorDomain::SearchMeta, "Searching"),
        (ErrorDomain::SearchFiles, "Searching"),
        (ErrorDomain::SearchInMeta, "Searching"),
        (ErrorDomain::SearchInPackageFiles, "Searching"),
    ];

    for (domain, expected) in cases {
        assert_eq!(StageName::new(domain, 0).to_string(), expected);
    }
}

#[test]
fn stage_name_from_c_error_matches_new() {
    locale::init_for_test();

    let error = CError {
        domain: ErrorDomain::Gc,
        state: 1,
        error: ErrorKind::Unexpected,
    };

    assert_eq!(StageName::from(&error).to_string(), "Collecting roots");
}

#[test]
fn lib_error_display_covers_every_error_kind() {
    locale::init_for_test();

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
        let error = LibError {
            error: CError {
                domain: ErrorDomain::Install,
                state: 0,
                error: kind,
            },
        };

        assert_eq!(error.to_string(), format!("{expected} (Install: Pre-hooks)"));
    }
}
