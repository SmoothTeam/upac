// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::io::Read;

use upac_abi::decoder::DecodeError;

use crate::{Dependency, PackageMeta};

pub fn read_to_string<R: Read>(reader: &mut R) -> Result<String, DecodeError> {
    let mut bytes = Vec::new();
    reader.read_to_end(&mut bytes)?;

    String::from_utf8(bytes).map_err(|_| DecodeError::InvalidUtf8)
}

#[derive(Debug)]
pub struct DecodedMeta {
    pub meta: PackageMeta,
    pub dependencies: Vec<Dependency>,
}

pub trait DecodeMeta {
    fn decode(&self, sha256: [u8; 32]) -> Result<DecodedMeta, DecodeError>;
}
