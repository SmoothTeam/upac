// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::os::raw::c_void;

use upac_macro::{CFree, CNew, CValidate};

use super::HookMessageFn;

use crate::hook::CancelToken;

pub mod booter;
pub mod bootstrap;
pub mod decoder;
pub mod format;
pub mod mutated;
pub mod partition;
pub mod unmutated;

#[repr(C)]
#[derive(Clone, Copy, CFree, CNew, CValidate)]
pub struct CRequestBase {
    pub struct_size: usize,

    pub on_hook: Option<HookMessageFn>,
    pub hook_ctx: *mut c_void,

    pub cancel_token: *mut CancelToken,
}
