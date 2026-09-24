// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: GPL-3.0-only

use std::env::temp_dir;
use std::sync::Arc;

use super::libcore::Lib;

pub mod abi;
pub mod errors;
pub mod progress;

macro_rules! boot_plugin {
    ($boot:expr) => {
        $boot
            .or_else(|| ::upac_types::settings::RuntimeSettings::load().boot.plugin)
            .ok_or_else(|| ::anyhow::anyhow!(::i18n_embed_fl::fl!($crate::locale::LOADER, "err-boot-plugin-required")))
    };
}
pub(crate) use boot_plugin;

macro_rules! request_base {
    () => {
        ::upac_types::request::RequestBase {
            on_hook: None,
            hook_ctx: ::std::ptr::null_mut(),
            cancel_token: $crate::cancel_token_ptr(),
        }
    };
    ($progress:expr) => {
        ::upac_types::request::RequestBase {
            on_hook: Some($crate::types::progress::on_progress),
            hook_ctx: $progress.ctx_ptr(),
            cancel_token: $crate::cancel_token_ptr(),
        }
    };
}
pub(crate) use request_base;

macro_rules! query {
    (@respond $symbol:expr, $request:expr, |$response:ident| $convert:expr) => {{
        let request = $request.into();
        let result = $crate::types::abi::invoke_with_response(|out, error| unsafe { ($symbol)(request, out, error) });
        unsafe { request.free() };

        result.and_then(|$response| {
            let converted = $convert;
            unsafe { $response.free() };
            converted
        })
    }};
    ($symbol:expr, $request:expr, strict $field:ident) => {
        $crate::types::query!(@respond $symbol, $request, |response| {
            ::std::vec::Vec::try_from(&response.$field)
                .map_err(|_| ::anyhow::anyhow!(::i18n_embed_fl::fl!($crate::locale::LOADER, "err-invalid-entry")))
        })
    };
    ($symbol:expr, $request:expr, $field:ident) => {
        $crate::types::query!(@respond $symbol, $request, |response| {
            ::anyhow::Ok(::std::vec::Vec::try_from(&response.$field).unwrap_or_default())
        })
    };
    ($symbol:expr, $request:expr, $($field:ident),+) => {
        $crate::types::query!(@respond $symbol, $request, |response| {
            ::anyhow::Ok(($(::std::vec::Vec::try_from(&response.$field).unwrap_or_default()),+))
        })
    };
}
pub(crate) use query;

macro_rules! call {
    ($symbol:expr, $request:expr) => {{
        let request = $request.into();
        let result = $crate::types::abi::invoke(|error| unsafe { ($symbol)(request, error) });
        unsafe { request.free() };
        result
    }};
}
pub(crate) use call;

pub struct CommandContext {
    pub lib: Arc<Lib>,
    pub tmp_path: String,
}

impl CommandContext {
    pub fn new(lib: Arc<Lib>) -> CommandContext {
        let tmp_path = temp_dir().to_string_lossy().into_owned();

        CommandContext { lib, tmp_path }
    }
}
