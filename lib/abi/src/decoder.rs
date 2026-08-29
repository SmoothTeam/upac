// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_macro::{CNew, CValidate};

use crate::error::ErrorKind;
use crate::hook::CancelToken;
use crate::package::{CPackageMeta, CVersion};
use crate::types::{CSlice, CVec, check_size};

pub const CONSTRAINT_LESS: u8 = 0b001;
pub const CONSTRAINT_EQUAL: u8 = 0b010;
pub const CONSTRAINT_GREATER: u8 = 0b100;
pub const CONSTRAINT_ANY: u8 = CONSTRAINT_LESS | CONSTRAINT_EQUAL | CONSTRAINT_GREATER;

pub type AbiVersionFn = unsafe extern "C" fn() -> u32;

pub type DecodeFn = unsafe extern "C" fn(request: *const CDecodeRequest, response_out: *mut CDecodeResponse) -> i32;

pub type FreeDecodeResponseFn = unsafe extern "C" fn(response: *mut CDecodeResponse);

/// Matches `token` against `operators`, longest operator first, returning the matched
/// constraint bitflags and how many bytes the operator itself consumed. Each package format
/// declares its own `operators` table (dependency version-constraint syntax differs per format —
/// e.g. alpm has bare `<`/`>`, deb only has `<<`/`>>`), so this only supplies the shared
/// longest-prefix-match logic, not any fixed operator set.
pub fn parse_constraint_prefix(token: &[u8], operators: &[(&[u8], u8)]) -> Option<(u8, usize)> {
    operators
        .iter()
        .find(|(operator, _)| token.starts_with(operator))
        .map(|(operator, constraint)| (*constraint, operator.len()))
}

#[repr(C)]
#[derive(CNew)]
pub struct CDecodeRequest {
    pub struct_size: usize,

    pub package_path: CSlice,
    pub output_dir: CSlice,

    pub checksum: [u8; 32],

    pub cancel_token: *mut CancelToken,
}

#[repr(C)]
#[derive(CValidate)]
pub struct CDecodeResponse {
    pub struct_size: usize,

    pub meta: CPackageMeta,

    pub dependencies: CVec<CDependency>,
    pub declarative_triggers: CVec<CSlice>,

    pub free: FreeDecodeResponseFn,
}

impl Drop for CDecodeResponse {
    fn drop(&mut self) {
        unsafe { (self.free)(self) };
    }
}

#[repr(C)]
#[derive(CValidate)]
pub struct CDependency {
    pub struct_size: usize,

    pub name: CSlice,
    pub constraint: u8,
    pub version: CVersion,
}
