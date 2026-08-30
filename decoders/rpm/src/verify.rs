// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::fs::File;
use std::io::{BufReader, Read};

use sha2::{Digest, Sha256};

use upac_abi::decoder::DecodeError;
use upac_abi::hook::CancelToken;

const READ_CHUNK_SIZE: usize = 65536;

pub fn verify(package_path: &str, expected_checksum: [u8; 32], cancel: &CancelToken) -> Result<(), DecodeError> {
    let file = File::open(package_path)?;
    let mut reader = BufReader::new(file);

    let mut hasher = Sha256::new();
    let mut buffer = [0u8; READ_CHUNK_SIZE];

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
