use std::os::raw::c_void;
use std::sync::atomic::{AtomicU8, Ordering};

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HookAck {
    Delivered = 0,
    Retry = 1,
}

pub trait MessageHook {
    fn send(&self, event: u32, data: *const c_void) -> HookAck;
}

pub trait CancelHook {
    fn is_cancelled(&self) -> bool;
    fn reset(&self);
}

pub type HookMessageFn = unsafe extern "C" fn(event: u32, data: *const c_void, ctx: *mut c_void) -> HookAck;

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
    fn send(&self, event: u32, data: *const c_void) -> HookAck {
        let Some(hook_message) = self.hook_message else {
            return HookAck::Delivered;
        };

        unsafe { hook_message(event, data, self.hook_message_context) }
    }
}

#[repr(C)]
pub struct HookCancelToken {
    cancelled: AtomicU8,
}

unsafe impl Sync for HookCancelToken {}

impl HookCancelToken {
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

pub struct Cancel {
    token: *const HookCancelToken,
}

impl Cancel {
    pub fn new(token: *const HookCancelToken) -> Self {
        Self { token }
    }
}

impl CancelHook for Cancel {
    fn is_cancelled(&self) -> bool {
        unsafe { (*self.token).is_cancelled() }
    }

    fn reset(&self) {
        unsafe { (*self.token).reset() };
    }
}

#[repr(C)]
pub struct CHookPreInstall {
    pub packages_count: u32,
    pub required_space: u64,
    pub free_space: u64,
}
