// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later

use std::collections::HashMap;
use std::fs::{File, create_dir_all, remove_dir_all};
use std::io::Read;
use std::path::Path;

use sha2::{Digest, Sha256};

use upac_abi::hook::CancelToken;

use upac_types::PackageTemp;

use crate::layout::decoders;
use crate::plugin::decoder::Decoder;
use crate::plugin::decoder::error::DecoderError;
use crate::plugin::decoder::manifest::{DecoderManifest, load_decoder_manifests};

pub struct PackageUnpacker {
    manifests: HashMap<String, DecoderManifest>,
    decoders: HashMap<String, Decoder>,
}

impl PackageUnpacker {
    pub fn new() -> Result<Self, DecoderError> {
        let manifests = load_decoder_manifests(decoders::DECODERS_DIR, decoders::MANIFEST_EXTENSION)?;

        Ok(Self {
            manifests,
            decoders: HashMap::new(),
        })
    }

    pub fn unpack_all(
        &mut self, package_paths: &[String], tmp_path: &str, cancel: &CancelToken,
    ) -> Result<Vec<PackageTemp>, DecoderError> {
        let mut packages = Vec::with_capacity(package_paths.len());
        let mut output_dirs = Vec::with_capacity(package_paths.len());

        for (index, package_path) in package_paths.iter().enumerate() {
            match self.unpack_one(package_path, index, tmp_path, cancel) {
                Ok(package) => {
                    output_dirs.push(package.temp_package_path.clone());
                    packages.push(package);
                }
                Err(error) => {
                    for output_dir in output_dirs.into_iter().rev() {
                        let _ = remove_dir_all(output_dir);
                    }

                    return Err(error);
                }
            }
        }

        Ok(packages)
    }

    fn unpack_one(
        &mut self, package_path: &str, index: usize, tmp_path: &str, cancel: &CancelToken,
    ) -> Result<PackageTemp, DecoderError> {
        let format = self.format_for(package_path)?;
        let checksum = checksum_of_file(package_path)?;

        let output_dir = format!("{tmp_path}/pkg-{index}");
        create_dir_all(&output_dir)?;

        let decoder = self.decoder_for(&format)?;
        let decoded = decoder
            .decode(package_path, &output_dir, checksum, cancel)
            .inspect_err(|_| {
                let _ = remove_dir_all(&output_dir);
            })?;

        Ok(PackageTemp {
            meta: decoded.meta,
            temp_package_path: output_dir,
        })
    }

    fn format_for(&self, package_path: &str) -> Result<String, DecoderError> {
        let extension = Path::new(package_path)
            .extension()
            .and_then(|extension| extension.to_str())
            .ok_or_else(|| DecoderError::UnknownFormat(package_path.to_owned()))?;

        self.manifests
            .values()
            .find(|manifest| manifest.extensions.iter().any(|candidate| candidate == extension))
            .map(|manifest| manifest.format.clone())
            .ok_or_else(|| DecoderError::UnknownFormat(package_path.to_owned()))
    }

    fn decoder_for(&mut self, format: &str) -> Result<&Decoder, DecoderError> {
        if !self.decoders.contains_key(format) {
            let manifest = self
                .manifests
                .get(format)
                .ok_or_else(|| DecoderError::UnknownFormat(format.to_owned()))?;
            let decoder = Decoder::load(&manifest.library)?;
            self.decoders.insert(format.to_owned(), decoder);
        }

        Ok(&self.decoders[format])
    }
}

fn checksum_of_file(path: &str) -> Result<[u8; 32], DecoderError> {
    let mut file = File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 65536];

    loop {
        let bytes_read = file.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    Ok(hasher.finalize().into())
}
