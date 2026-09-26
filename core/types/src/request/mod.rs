// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::os::raw::c_void;

use upac_abi::error::ErrorKind;
use upac_abi::hook::CancelToken;
use upac_abi::hook::HookMessageFn;
use upac_abi::request::CRequestBase;

use upac_macro::{CTryToRust, RustToC};

use crate::hook::Message;
use crate::traits::MessageHook;

pub mod booter;
pub mod bootstrap;
pub mod decoder;
pub mod format;
pub mod mutated;
pub mod partition;
pub mod unmutated;

#[derive(Debug, Clone, RustToC, CTryToRust)]
pub struct RequestBase {
    pub on_hook: Option<HookMessageFn>,
    pub hook_ctx: *mut c_void,
    pub cancel_token: *mut CancelToken,
}

impl RequestBase {
    pub fn message_hook(&self) -> Box<dyn MessageHook> {
        Box::new(Message::new(self.on_hook, self.hook_ctx))
    }
}
