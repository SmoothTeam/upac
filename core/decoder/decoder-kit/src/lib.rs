// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::fs::File;
use std::io::{BufReader, Read};

use sha2::{Digest, Sha256};

use upac_abi::package::{CPackageDependency, CPackageMeta, CPackageTrigger};
use upac_abi::response::decoder::CDecodeResponse;
use upac_abi::types::{COwned, CVec};

use upac_types::CancelToken;
use upac_types::decoder::{DecodeError, PackageTrigger};
use upac_types::package::{DecodedPackageMeta, VersionConstraint};

const VERIFY_CHUNK_SIZE: usize = 65536;

/// # Safety
/// `response`, if non-null, must point to a `CDecodeResponse` built by `build_decode_response` in
/// this same plugin, not yet freed.
pub unsafe extern "C" fn free_decode_response(response: *mut CDecodeResponse) {
    if response.is_null() {
        return;
    }

    unsafe { (&*response).free() };
}

pub fn parse_constraint_prefix(
    token: &[u8], operators: &[(&[u8], VersionConstraint)],
) -> Option<(VersionConstraint, usize)> {
    operators
        .iter()
        .find(|(operator, _)| token.starts_with(operator))
        .map(|(operator, constraint)| (*constraint, operator.len()))
}

pub fn read_to_string<R: Read>(reader: &mut R) -> Result<String, DecodeError> {
    let mut bytes = Vec::new();
    reader.read_to_end(&mut bytes)?;

    String::from_utf8(bytes).map_err(|_| DecodeError::InvalidUtf8)
}

pub fn verify(package_path: &str, expected_checksum: [u8; 32], cancel: &CancelToken) -> Result<(), DecodeError> {
    let file = File::open(package_path)?;
    let mut reader = BufReader::new(file);

    let mut hasher = Sha256::new();
    let mut buffer = [0u8; VERIFY_CHUNK_SIZE];

    loop {
        if cancel.is_cancelled() {
            return Err(DecodeError::Cancelled);
        }

        let bytes_read = reader.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }

        hasher.update(&buffer[..bytes_read]);
    }

    if hasher.finalize().as_slice() != expected_checksum.as_slice() {
        return Err(DecodeError::ChecksumMismatch);
    }

    Ok(())
}

pub fn build_decode_response(decoded: DecodedPackageMeta, triggers: Vec<PackageTrigger>) -> CDecodeResponse {
    let DecodedPackageMeta { meta, dependencies } = decoded;

    let dependencies = dependencies
        .into_iter()
        .map(CPackageDependency::from)
        .collect::<Vec<_>>();

    let triggers = triggers.into_iter().map(CPackageTrigger::from).collect::<Vec<_>>();

    CDecodeResponse::new(
        CPackageMeta::from(meta),
        CVec::from_owned(dependencies),
        CVec::from_owned(triggers),
        free_decode_response,
    )
}
