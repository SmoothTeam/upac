// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::mem::MaybeUninit;

use upac_abi::plugin::DecodeFn;
use upac_abi::request::decoder::CDecodeRequest;
use upac_abi::response::decoder::CDecodeResponse;

use upac_types::CancelToken;
use upac_types::plugin::plugin_result;
use upac_types::request::decoder::DecodeRequest;
use upac_types::response::decoder::DecodeResponse;

#[cfg(feature = "dynamic-plugins")]
use libloading::Library;

use self::error::DecoderError;

#[cfg(feature = "dynamic-plugins")]
mod dynamic_link;
mod layout {
    include!(concat!(env!("OUT_DIR"), "/layout.rs"));
}
#[cfg(feature = "builtin-decoders")]
mod static_link;

pub mod error;
pub mod manifest;
pub mod unpack;

pub struct DecoderPlugin {
    decode: DecodeFn,

    #[cfg(feature = "dynamic-plugins")]
    _library: Option<Library>,
}

impl DecoderPlugin {
    pub fn decode(
        &self, package_path: &str, output_dir: &str, checksum: [u8; 32], cancel: &CancelToken,
    ) -> Result<DecodeResponse, DecoderError> {
        let request: CDecodeRequest = DecodeRequest {
            package_path,
            output_dir,
            checksum,
            cancel_token: cancel,
        }
        .into();

        let mut response = MaybeUninit::<CDecodeResponse>::uninit();

        let status = unsafe { (self.decode)(&request, response.as_mut_ptr()) };
        unsafe { request.free() };

        plugin_result(status).map_err(|error| error.map_or(DecoderError::InvalidResponse, DecoderError::Failed))?;

        let response = unsafe { response.assume_init() };

        Ok(DecodeResponse::try_from(&response)?)
    }
}
