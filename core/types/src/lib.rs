// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::sync::atomic::{AtomicU8, Ordering};

use self::error::ErrorKind;

pub mod booter;
pub mod decoder;
pub mod diff;
pub mod error;
pub mod package;
pub mod plugin;
pub mod progress;
pub mod request;
pub mod response;
pub mod settings;
pub mod state;
pub mod traits;
pub mod transaction;

#[repr(transparent)]
#[derive(Debug, Default)]
pub struct CancelToken(AtomicU8);

impl CancelToken {
    pub const fn new() -> Self {
        CancelToken(AtomicU8::new(0))
    }

    pub fn cancel(&self) {
        self.0.store(1, Ordering::Release);
    }

    pub fn is_cancelled(&self) -> bool {
        self.0.load(Ordering::Acquire) != 0
    }

    fn as_ptr(&self) -> *const AtomicU8 {
        &self.0
    }

    unsafe fn from_ptr<'token>(pointer: *const AtomicU8) -> Result<&'token CancelToken, ErrorKind> {
        unsafe { pointer.cast::<CancelToken>().as_ref() }.ok_or(ErrorKind::InvalidEntry)
    }
}
