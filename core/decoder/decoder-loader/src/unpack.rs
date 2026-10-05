// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_types::CancelToken;
use upac_types::decoder::PackageTriggers;
use upac_types::package::PackageMeta;

use super::error::DecoderError;

#[cfg(any(feature = "dynamic-plugins", feature = "builtin-decoders"))]
use std::fs::{File, create_dir_all, remove_dir_all};
#[cfg(any(feature = "dynamic-plugins", feature = "builtin-decoders"))]
use std::io::Read;
#[cfg(any(feature = "dynamic-plugins", feature = "builtin-decoders"))]
use std::path::Path;

#[cfg(any(feature = "dynamic-plugins", feature = "builtin-decoders"))]
use sha2::{Digest, Sha256};

#[cfg(any(feature = "dynamic-plugins", feature = "builtin-decoders"))]
use std::collections::HashMap;

#[cfg(any(feature = "dynamic-plugins", feature = "builtin-decoders"))]
use super::DecoderPlugin;

#[cfg(feature = "dynamic-plugins")]
use super::dynamic_link::load_decoder_dynamic;
#[cfg(feature = "dynamic-plugins")]
use super::manifest::DecoderManifests;

#[cfg(feature = "builtin-decoders")]
use super::BUILTIN_DECODERS;

#[cfg(all(test, any(feature = "dynamic-plugins", feature = "builtin-decoders")))]
#[path = "../tests/inline/unpack.rs"]
mod tests;

#[derive(Debug, Clone)]
pub struct PackageTemp {
    pub meta: PackageMeta,
    pub temp_package_path: String,
}

#[cfg(any(feature = "dynamic-plugins", feature = "builtin-decoders"))]
fn has_extension(file_name: &str, extension: &str) -> bool {
    file_name
        .strip_suffix(extension)
        .is_some_and(|stem| stem.ends_with('.'))
}

#[cfg(any(feature = "dynamic-plugins", feature = "builtin-decoders"))]
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

pub struct PackageUnpacker {
    #[cfg(feature = "dynamic-plugins")]
    manifests: DecoderManifests,

    #[cfg(any(feature = "dynamic-plugins", feature = "builtin-decoders"))]
    decoders: HashMap<String, DecoderPlugin>,
}

#[cfg(any(feature = "dynamic-plugins", feature = "builtin-decoders"))]
impl PackageUnpacker {
    pub fn unpack_one(
        &mut self, package_path: &str, index: usize, tmp_path: &str, cancel: &CancelToken,
    ) -> Result<(PackageTemp, PackageTriggers), DecoderError> {
        let format = self.format_for(package_path)?;
        let checksum = checksum_of_file(package_path)?;

        let output_dir = format!("{tmp_path}/pkg-{index}");
        create_dir_all(&output_dir)?;

        let decoder = self.decoder_for(&format).inspect_err(|_| {
            let _ = remove_dir_all(&output_dir);
        })?;

        let decoded = decoder
            .decode(package_path, &output_dir, checksum, cancel)
            .inspect_err(|_| {
                let _ = remove_dir_all(&output_dir);
            })?;

        Ok((
            PackageTemp {
                meta: decoded.meta,
                temp_package_path: output_dir,
            },
            PackageTriggers {
                format,
                triggers: decoded.triggers,
            },
        ))
    }
}

#[cfg(any(feature = "dynamic-plugins", feature = "builtin-decoders"))]
impl PackageUnpacker {
    pub fn new() -> Result<Self, DecoderError> {
        Ok(Self {
            #[cfg(feature = "dynamic-plugins")]
            manifests: DecoderManifests::new()?,

            decoders: HashMap::new(),
        })
    }

    fn format_for(&self, package_path: &str) -> Result<String, DecoderError> {
        let file_name = Path::new(package_path)
            .file_name()
            .and_then(|file_name| file_name.to_str())
            .ok_or_else(|| DecoderError::UnknownFormat(package_path.to_owned()))?;

        #[cfg(feature = "builtin-decoders")]
        if let Some(builtin) = BUILTIN_DECODERS.iter().find(|builtin| {
            builtin
                .extensions
                .iter()
                .any(|extension| has_extension(file_name, extension))
        }) {
            return Ok(builtin.format.to_owned());
        }

        #[cfg(feature = "dynamic-plugins")]
        if let Some(manifest) = self.manifests.0.values().find(|manifest| {
            manifest
                .extensions
                .iter()
                .any(|extension| has_extension(file_name, extension))
        }) {
            return Ok(manifest.format.clone());
        }

        Err(DecoderError::UnknownFormat(package_path.to_owned()))
    }

    fn decoder_for(&mut self, format: &str) -> Result<&DecoderPlugin, DecoderError> {
        if !self.decoders.contains_key(format) {
            let decoder = self.load_decoder(format)?;
            self.decoders.insert(format.to_owned(), decoder);
        }

        Ok(&self.decoders[format])
    }

    fn load_decoder(&self, format: &str) -> Result<DecoderPlugin, DecoderError> {
        #[cfg(feature = "builtin-decoders")]
        if let Some(builtin) = BUILTIN_DECODERS.iter().find(|builtin| builtin.format == format) {
            return Ok(DecoderPlugin::from(builtin));
        }

        self.load_dynamic_decoder(format)
    }

    #[cfg(feature = "dynamic-plugins")]
    fn load_dynamic_decoder(&self, format: &str) -> Result<DecoderPlugin, DecoderError> {
        load_decoder_dynamic(&self.manifests, format)
    }

    #[cfg(not(feature = "dynamic-plugins"))]
    fn load_dynamic_decoder(&self, format: &str) -> Result<DecoderPlugin, DecoderError> {
        Err(DecoderError::UnknownFormat(format.to_owned()))
    }
}

#[cfg(all(not(feature = "dynamic-plugins"), not(feature = "builtin-decoders")))]
impl PackageUnpacker {
    pub fn new() -> Result<Self, DecoderError> {
        Err(DecoderError::NoDecoders)
    }

    pub fn unpack_one(
        &mut self, _package_path: &str, _index: usize, _tmp_path: &str, _cancel: &CancelToken,
    ) -> Result<(PackageTemp, PackageTriggers), DecoderError> {
        Err(DecoderError::NoDecoders)
    }
}
