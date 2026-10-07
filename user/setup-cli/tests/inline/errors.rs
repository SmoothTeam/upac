// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: GPL-3.0-only

use upac_types::error::{Error as AbiError, ErrorDomain, ErrorKind};
use upac_types::state::setup::{BootstrapImportStateId, FormatStateId, PartitionAddStateId, PartitionTableStateId};

use crate::locale;

use super::{AbiMismatch, LibError};

fn localized(state: BootstrapImportStateId, kind: ErrorKind) -> String {
    locale::init_for_test();

    let error = AbiError {
        domain: ErrorDomain::BootstrapImport,
        state: state as u32,
        kind,
    };

    LibError(error).to_string()
}

#[test]
fn prefixes_the_message_with_the_localized_failing_stage_name() {
    let message = localized(BootstrapImportStateId::Add, ErrorKind::Unexpected);

    assert_eq!(message, "Adding package: Unexpected error");
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
        (
            ErrorKind::NotAPartition,
            "Not a partition: pass a partition such as /dev/vda1, not a whole disk",
        ),
        (
            ErrorKind::WrongPartitionType,
            "The partition has a different GPT type than required",
        ),
        (
            ErrorKind::UnsupportedFilesystem,
            "No supported filesystem on the partition: format it first (up-sp format create)",
        ),
        (
            ErrorKind::ToolNotInstalled,
            "A required program is not installed on this system (e.g. dracut or mkinitcpio for --initramfs-generator)",
        ),
        (
            ErrorKind::ToolFailed,
            "An external program failed; see its output above",
        ),
        (
            ErrorKind::RollbackFailed,
            "The operation failed and undoing its partial changes failed too; the system may be left inconsistent",
        ),
    ];

    for (kind, expected) in cases {
        let message = localized(BootstrapImportStateId::Setup, kind);

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
