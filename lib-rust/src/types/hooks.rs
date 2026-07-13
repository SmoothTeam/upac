use std::os::raw::c_void;

use crate::ffi::{HookAck, HookCancelToken, HookMessageFn};

pub struct HookMessageHandle {
    hook_message: Option<HookMessageFn>,
    hook_message_context: *mut c_void,
}

impl HookMessageHandle {
    pub fn new(hook_message: Option<HookMessageFn>, hook_message_context: *mut c_void) -> Self {
        Self {
            hook_message,
            hook_message_context,
        }
    }

    pub fn call(&self, event: u32, data: *const c_void) {
        let Some(hook_message) = self.hook_message else { return };

        while unsafe { hook_message(event, data, self.hook_message_context) } == HookAck::Retry {}
    }
}

pub struct HookCancelHandle {
    token: *const HookCancelToken,
}

impl HookCancelHandle {
    pub fn new(token: *const HookCancelToken) -> Self {
        Self { token }
    }

    pub fn is_cancelled(&self) -> bool {
        unsafe { (*self.token).is_cancelled() }
    }
}
