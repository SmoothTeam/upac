// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::ffi::FromBytesWithNulError;
use std::mem::size_of;
use std::str::Utf8Error;

use upac_macro::{CFree, CValidate};

use crate::types::CSlice;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AbiError {
    InvalidEntry,
    AbiMismatch,
}

impl From<FromBytesWithNulError> for AbiError {
    fn from(_: FromBytesWithNulError) -> Self {
        AbiError::InvalidEntry
    }
}

impl From<Utf8Error> for AbiError {
    fn from(_: Utf8Error) -> Self {
        AbiError::InvalidEntry
    }
}

#[repr(C)]
#[derive(Clone, Copy, CFree, CValidate)]
pub struct CError {
    pub struct_size: usize,

    pub domain: u32,
    pub state: u32,
    pub kind: u32,
    #[optional]
    pub subject: CSlice,
}

impl Default for CError {
    fn default() -> Self {
        CError {
            struct_size: size_of::<CError>(),
            domain: 0,
            state: 0,
            kind: 0,
            subject: CSlice::from_slice(None),
        }
    }
}
