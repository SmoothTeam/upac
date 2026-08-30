// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::fs::File;
use std::io::{Cursor, Read};

use flate2::read::GzDecoder;
use tar::Archive;
use xz2::read::XzDecoder;
use zstd::stream::read::Decoder as ZstdDecoder;

use upac_abi::decoder::DecodeError;
use upac_abi::hook::CancelToken;
use upac_types::decoder::read_to_string;

use crate::xbps::{FILES_ENTRY, INSTALL_ENTRY, PROPS_ENTRY, REMOVE_ENTRY};

const ZSTD_MAGIC: [u8; 4] = [0x28, 0xB5, 0x2F, 0xFD];
const XZ_MAGIC: [u8; 6] = [0xFD, b'7', b'z', b'X', b'Z', 0x00];
const GZIP_MAGIC: [u8; 2] = [0x1F, 0x8B];

pub struct ExtractedMetadata {
    pub props: String,
    pub install_present: bool,
    pub remove_present: bool,
}

impl ExtractedMetadata {
    pub fn extract(package_path: &str, output_dir: &str, cancel: &CancelToken) -> Result<Self, DecodeError> {
        let file = File::open(package_path)?;
        let reader = Self::open_reader(file)?;

        let mut archive = Archive::new(reader);

        let mut props = None;
        let mut install_present = false;
        let mut remove_present = false;

        for entry in archive.entries()? {
            if cancel.is_cancelled() {
                return Err(DecodeError::Cancelled);
            }

            let mut entry = entry?;
            let entry_path = entry.path()?.to_string_lossy().into_owned();
            let entry_name = entry_path.strip_prefix("./").unwrap_or(&entry_path);

            match entry_name {
                PROPS_ENTRY => {
                    props = Some(read_to_string(&mut entry)?);
                    continue;
                }
                FILES_ENTRY => continue,
                INSTALL_ENTRY => {
                    install_present = true;
                    continue;
                }
                REMOVE_ENTRY => {
                    remove_present = true;
                    continue;
                }
                _ => {}
            }

            entry.unpack_in(output_dir)?;
        }

        props
            .map(|props| ExtractedMetadata {
                props,
                install_present,
                remove_present,
            })
            .ok_or(DecodeError::MissingMetadata)
    }

    /// `.xbps` filenames carry no compression suffix (unlike alpm's `.pkg.tar.{zst,xz,gz}`) — the
    /// compression filter is sniffed from the file's own magic bytes instead.
    fn open_reader(mut file: File) -> Result<Box<dyn Read>, DecodeError> {
        let mut magic = [0u8; 6];
        let bytes_read = file.read(&mut magic)?;
        let sniffed = &magic[..bytes_read];
        let chained = Cursor::new(sniffed.to_vec()).chain(file);

        match () {
            _ if sniffed.starts_with(&ZSTD_MAGIC) => Ok(Box::new(ZstdDecoder::new(chained)?)),
            _ if sniffed.starts_with(&XZ_MAGIC) => Ok(Box::new(XzDecoder::new(chained))),
            _ if sniffed.starts_with(&GZIP_MAGIC) => Ok(Box::new(GzDecoder::new(chained))),
            _ => Ok(Box::new(chained)),
        }
    }
}
