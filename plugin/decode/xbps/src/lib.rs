// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::path::Path;

use upac_abi::DECODER_ABI_VERSION;
use upac_abi::request::CDecodeRequest;
use upac_abi::response::CDecodeResponse;

use upac_decoder_kit::ObjectStore;

use upac_types::DecodedPackageMeta;
use upac_types::decoder::{DecodeError, DecodeMeta, plugin_status};
use upac_types::request::DecodeRequest;
use upac_types::response::DecodeResponse;

use self::archive::XbpsArchive;
use self::meta::Props;

mod archive;
mod layout {
    include!(concat!(env!("OUT_DIR"), "/layout.rs"));
}

pub mod meta;
pub mod triggers;

#[unsafe(no_mangle)]
pub extern "C" fn decode_abi_version() -> u32 {
    DECODER_ABI_VERSION
}

/// # Safety
/// `request`, if non-null, must point to a valid, initialized `CDecodeRequest` for the duration
/// of the call. `response_out`, if non-null, must point to writable, uninitialized
/// `CDecodeResponse` storage that this function fully initializes on success.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn decode(request: *const CDecodeRequest, response_out: *mut CDecodeResponse) -> i32 {
    fn decode_package(request: &CDecodeRequest) -> Result<CDecodeResponse, DecodeError> {
        let request = DecodeRequest::try_from(request)?;
        let package_path = Path::new(request.package_path);

        request.checksum.verify(package_path, request.cancel_token)?;

        let objects = ObjectStore::new(request.create_file, request.finish_file, request.store_ctx);
        let archive = XbpsArchive::read(package_path, &objects, request.cancel_token)?;

        let DecodedPackageMeta { meta, dependencies } = Props(&archive.props).decode(request.checksum)?;
        let triggers = triggers::scan(archive.install_present, archive.remove_present);

        Ok(DecodeResponse {
            meta,
            dumpfile: archive.dumpfile,
            dependencies,
            triggers,
        }
        .into())
    }

    if request.is_null() || response_out.is_null() {
        return plugin_status(Err(DecodeError::InvalidRequest));
    }

    let result = decode_package(unsafe { &*request }).map(|response| unsafe { response_out.write(response) });

    plugin_status(result)
}
