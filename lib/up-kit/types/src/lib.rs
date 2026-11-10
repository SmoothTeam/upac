// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::sync::atomic::{AtomicU8, Ordering as AtomicOrdering};

use upac_abi::CProgressEvent;

use upac_macro::{CEnum, RustToC};

use self::error::ErrorKind;

pub mod booter;
pub mod decoder;
pub mod entry;
pub mod error;
pub mod export;
pub mod package;
pub mod request;
pub mod settings;
pub mod state;
pub mod transaction;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, CEnum)]
pub enum FileDiffKind {
    Added = 0,
    Removed = 1,
    Modified = 2,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, CEnum)]
pub enum PackageDiffKind {
    Added = 0,
    Removed = 1,
    Modified = 2,
    FilesChanged = 3,
}

#[repr(transparent)]
#[derive(Debug, Default)]
pub struct CancelToken(AtomicU8);

impl CancelToken {
    pub const fn new() -> Self {
        CancelToken(AtomicU8::new(0))
    }

    pub fn cancel(&self) {
        self.0.store(1, AtomicOrdering::Release);
    }

    pub fn is_cancelled(&self) -> bool {
        self.0.load(AtomicOrdering::Acquire) != 0
    }

    fn as_ptr(&self) -> *const AtomicU8 {
        &self.0
    }

    unsafe fn from_ptr<'token>(pointer: *const AtomicU8) -> Result<&'token CancelToken, ErrorKind> {
        unsafe { pointer.cast::<CancelToken>().as_ref() }.ok_or(ErrorKind::InvalidEntry)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, RustToC)]
pub struct ProgressEvent<'event> {
    pub stage: u32,
    pub subject: Option<&'event str>,
    pub current: u64,
    pub total: u64,
}
