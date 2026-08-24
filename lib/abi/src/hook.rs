// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later

use std::mem::size_of;
use std::os::raw::c_void;
use std::sync::atomic::{AtomicU8, Ordering};

use crate::types::CSlice;

pub type HookMessageFn = unsafe extern "C" fn(event: *const CProgressEvent, ctx: *mut c_void) -> HookAck;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HookAck {
    Delivered = 0,
    Retry = 1,
}

#[repr(C)]
pub struct CProgressEvent {
    pub struct_size: usize,
    pub stage: u32,
    pub phase: u32,
    pub subject: CSlice,
    pub current: u64,
    pub total: u64,
}

pub struct ProgressEventBuilder {
    stage: u32,
    phase: u32,
    subject: Option<String>,
    current: u64,
    total: u64,
}

impl ProgressEventBuilder {
    pub fn new(stage: u32) -> Self {
        Self {
            stage,
            phase: 0,
            subject: None,
            current: 0,
            total: 0,
        }
    }

    pub fn stage(&self) -> u32 {
        self.stage
    }

    pub fn phase(mut self, phase: u32) -> Self {
        self.phase = phase;
        self
    }

    pub fn subject(mut self, subject: impl Into<String>) -> Self {
        self.subject = Some(subject.into());
        self
    }

    pub fn progress(mut self, current: u64, total: u64) -> Self {
        self.current = current;
        self.total = total;
        self
    }

    pub fn build(&self) -> CProgressEvent {
        CProgressEvent {
            struct_size: size_of::<CProgressEvent>(),
            stage: self.stage,
            phase: self.phase,
            subject: CSlice::from_slice(self.subject.as_deref().map(str::as_bytes)),
            current: self.current,
            total: self.total,
        }
    }
}

pub trait MessageHook {
    fn send(&self, event: &CProgressEvent) -> HookAck;
}

pub struct Message {
    hook_message: Option<HookMessageFn>,
    hook_message_context: *mut c_void,
}

impl Message {
    pub fn new(hook_message: Option<HookMessageFn>, hook_message_context: *mut c_void) -> Self {
        Self {
            hook_message,
            hook_message_context,
        }
    }
}

impl MessageHook for Message {
    fn send(&self, event: &CProgressEvent) -> HookAck {
        let Some(hook_message) = self.hook_message else {
            return HookAck::Delivered;
        };

        unsafe { hook_message(event as *const CProgressEvent, self.hook_message_context) }
    }
}

#[repr(C)]
pub struct CancelToken {
    cancelled: AtomicU8,
}

unsafe impl Sync for CancelToken {}

impl CancelToken {
    pub const fn new() -> Self {
        Self {
            cancelled: AtomicU8::new(0),
        }
    }

    pub fn cancel(&self) {
        self.cancelled.store(1, Ordering::Release);
    }

    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::Acquire) != 0
    }

    pub fn reset(&self) {
        self.cancelled.store(0, Ordering::Release);
    }
}

impl Default for CancelToken {
    fn default() -> Self {
        Self::new()
    }
}

#[repr(C)]
pub struct CHookPreInstall {
    pub packages_count: u32,
    pub required_space: u64,
    pub free_space: u64,
}
