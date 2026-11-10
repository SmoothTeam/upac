// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: GPL-3.0-only

use upac_pki::error::PkiError;

use crate::locale;

use super::LocalizedPkiError;

#[test]
fn display_covers_every_pki_error_variant() {
    locale::init_for_test();

    let cases = [
        (PkiError::Malformed, "Malformed PKI data"),
        (PkiError::InvalidSignature, "Invalid signature"),
        (PkiError::Generation, "Certificate generation failed"),
    ];

    for (error, expected) in cases {
        assert_eq!(LocalizedPkiError(error).to_string(), expected);
    }
}
