// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 JustPav
//
// SPDX-License-Identifier: LGPL-3.0-or-later

use std::array::TryFromSliceError;

use der::Error as DerError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PkiError {
    Malformed,
    InvalidSignature,
}

impl From<DerError> for PkiError {
    fn from(_: DerError) -> Self {
        PkiError::Malformed
    }
}

impl From<TryFromSliceError> for PkiError {
    fn from(_: TryFromSliceError) -> Self {
        PkiError::Malformed
    }
}
