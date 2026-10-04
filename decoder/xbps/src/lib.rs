// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_abi::DECODER_ABI_VERSION;
use upac_abi::request::decoder::CDecodeRequest;
use upac_abi::response::decoder::CDecodeResponse;

use upac_decoder_kit::{build_decode_response, verify};
use upac_types::decoder::DecodeError;
use upac_types::plugin::plugin_status;
use upac_types::request::decoder::DecodeRequest;
use upac_types::traits::DecodeMeta;

use self::extract::ExtractedMetadata;
use self::meta::Props;
use self::triggers::scan;

pub mod meta;
pub mod triggers;

mod extract;

include!(concat!(env!("OUT_DIR"), "/layout.rs"));

/// # Safety
/// Touches no pointers.
#[cfg_attr(feature = "cdylib", unsafe(no_mangle))]
pub unsafe extern "C" fn decode_abi_version() -> u32 {
    DECODER_ABI_VERSION
}

/// # Safety
/// `request`, if non-null, must point to a valid, initialized `CDecodeRequest` for the duration
/// of the call. `response_out`, if non-null, must point to writable, uninitialized
/// `CDecodeResponse` storage that this function fully initializes on success.
#[cfg_attr(feature = "cdylib", unsafe(no_mangle))]
pub unsafe extern "C" fn decode(request: *const CDecodeRequest, response_out: *mut CDecodeResponse) -> i32 {
    if request.is_null() || response_out.is_null() {
        return plugin_status(Err(DecodeError::InvalidRequest));
    }

    let result = decode_package(unsafe { &*request }).map(|response| unsafe { response_out.write(response) });

    plugin_status(result)
}

fn decode_package(request: &CDecodeRequest) -> Result<CDecodeResponse, DecodeError> {
    let DecodeRequest {
        package_path,
        output_dir,
        checksum,
        cancel_token,
    } = DecodeRequest::try_from(request)?;

    verify(package_path, checksum, cancel_token)?;

    let extracted = ExtractedMetadata::extract(package_path, output_dir, cancel_token)?;
    let package_triggers = scan(extracted.install_present, extracted.remove_present);

    let decoded = Props(&extracted.props).decode(checksum)?;

    Ok(build_decode_response(decoded, package_triggers))
}
