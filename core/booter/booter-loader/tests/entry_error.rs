// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use anyhow::anyhow;

use upac_boot_loader::entry::error::BootEntryError;

use upac_types::error::ErrorKind;

#[test]
fn anyhow_error_maps_to_unexpected() {
    assert_eq!(BootEntryError::from(anyhow!("boom")), BootEntryError::Unexpected);
}

#[test]
fn every_variant_maps_to_the_documented_error_kind() {
    let cases = [
        (BootEntryError::NoBootResource, ErrorKind::NotFound),
        (BootEntryError::AmbiguousBootResource, ErrorKind::InvalidEntry),
        (BootEntryError::UnsupportedBootResource, ErrorKind::InvalidEntry),
        (BootEntryError::Unexpected, ErrorKind::Unexpected),
    ];

    for (error, expected) in cases {
        assert_eq!(ErrorKind::from(error), expected);
    }
}
