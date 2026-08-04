// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 JustPav
//
// SPDX-License-Identifier: LGPL-3.0-or-later

use upac_abi::error::ErrorKind;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecoderError {
    Load,
    Symbol,
    AbiMismatch { got: u32, expected: u32 },
    Failed(i32),
    InvalidResponse,
}

impl From<ErrorKind> for DecoderError {
    fn from(_: ErrorKind) -> Self {
        DecoderError::InvalidResponse
    }
}

impl From<DecoderError> for ErrorKind {
    fn from(error: DecoderError) -> Self {
        match error {
            DecoderError::Load => ErrorKind::NotFound,
            DecoderError::Symbol => ErrorKind::AbiMismatch,
            DecoderError::AbiMismatch { .. } => ErrorKind::AbiMismatch,
            DecoderError::Failed(_) => ErrorKind::Unexpected,
            DecoderError::InvalidResponse => ErrorKind::InvalidEntry,
        }
    }
}
