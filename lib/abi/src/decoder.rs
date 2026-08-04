// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 JustPav
//
// SPDX-License-Identifier: LGPL-3.0-or-later

use std::slice::from_raw_parts;

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

pub type MatchTriggersFn = unsafe extern "C" fn(table: *const CTriggerTable, matches: *mut CTriggerMatches) -> i32;

#[repr(C)]
pub struct CDecodeRequest {
    pub struct_size: usize,

    pub package_path: CSlice,
    pub output_dir: CSlice,

    pub checksum: [u8; 32],

    pub cancel_token: *mut CancelToken,
}

#[repr(C)]
pub struct CDecodeResponse {
    pub struct_size: usize,

    pub meta: CPackageMeta,

    pub dependencies: CVec<CDependency>,

    pub free: FreeDecodeResponseFn,
}

impl CDecodeResponse {
    /// # Safety
    /// `meta` and every `CDependency` in `dependencies` must be null/empty or point to valid,
    /// correctly sized memory for the duration of this call — this is the entry point that checks an
    /// untrusted, decoder-supplied struct is safe to read further.
    pub unsafe fn validate(&self) -> Result<(), ErrorKind> {
        check_size::<CDecodeResponse>(self.struct_size)?;
        unsafe { self.meta.validate()? };

        unsafe { self.dependencies.validate()? };

        for dependency in unsafe { self.dependencies.as_slice() } {
            unsafe { dependency.validate()? };
        }

        Ok(())
    }
}

impl Drop for CDecodeResponse {
    fn drop(&mut self) {
        unsafe { (self.free)(self) };
    }
}

#[repr(C)]
pub struct CDependency {
    pub struct_size: usize,

    pub name: CSlice,
    pub constraint: u8,
    pub version: CVersion,
}

impl CDependency {
    /// # Safety
    /// `name` must be null/empty or point to valid, correctly sized memory for the duration of this
    /// call — this is the entry point that checks an untrusted, decoder-supplied struct is safe to
    /// read further.
    pub unsafe fn validate(&self) -> Result<(), ErrorKind> {
        check_size::<CDependency>(self.struct_size)?;
        unsafe { self.name.validate()? };
        unsafe { self.version.validate()? };
        Ok(())
    }
}

#[repr(C)]
pub struct CTriggerEntry {
    pub struct_size: usize,
    pub name: CSlice,
    pub hook_id: u16,
}

impl CTriggerEntry {
    /// # Safety
    /// `name` must be null/empty or point to valid, correctly sized memory for the duration of this
    /// call — this is the entry point that checks an untrusted, C-supplied struct is safe to read
    /// further.
    pub unsafe fn validate(&self) -> Result<(), ErrorKind> {
        check_size::<CTriggerEntry>(self.struct_size)?;

        unsafe { self.name.validate()? };

        Ok(())
    }
}

#[repr(C)]
pub struct CTriggerTable {
    pub struct_size: usize,

    pub entries: CVec<CTriggerEntry>,
}

impl CTriggerTable {
    /// # Safety
    /// Every `CTriggerEntry` in `entries` must be null/empty or point to valid, correctly sized memory
    /// for the duration of this call — this is the entry point that checks an untrusted, C-supplied
    /// struct is safe to read further.
    pub unsafe fn validate(&self) -> Result<(), ErrorKind> {
        check_size::<CTriggerTable>(self.struct_size)?;

        unsafe { self.entries.validate()? };

        for entry in unsafe { self.entries.as_slice() } {
            unsafe { entry.validate()? };
        }

        Ok(())
    }
}

#[repr(C)]
pub struct CTriggerMatches {
    pub struct_size: usize,

    pub ids: *mut u16,
    pub capacity: usize,
    pub len: usize,
}

impl CTriggerMatches {
    /// # Safety
    /// `ids` must point to `capacity` writable `u16` slots, and `len` must have been written by the
    /// decoder (or left at `0`) before this is called.
    pub unsafe fn matched(&self) -> &[u16] {
        if self.ids.is_null() {
            return &[];
        }

        let len = self.len.min(self.capacity);

        unsafe { from_raw_parts(self.ids, len) }
    }
}
