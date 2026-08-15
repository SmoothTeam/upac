// SPDX-FileCopyrightText: 2026 JustPav
//
// SPDX-License-Identifier: GPL-3.0-only

use std::fmt::{Display, Formatter};

use upac_pki::error::PkiError;

#[derive(Debug)]
pub struct LocalizedPkiError(pub PkiError);

impl Display for LocalizedPkiError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        let key = match self.0 {
            PkiError::Malformed => "err_malformed",
            PkiError::InvalidSignature => "err_invalid_signature",
            PkiError::Generation => "err_generation",
        };
        formatter.write_str(&gettextrs::gettext(key))
    }
}

impl std::error::Error for LocalizedPkiError {}
