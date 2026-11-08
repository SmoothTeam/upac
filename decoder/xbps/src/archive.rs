// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::fs::File;
use std::io::{BufRead, BufReader, Read, read_to_string};
use std::path::Path;

use flate2::bufread::GzDecoder;
use tar::Archive;
use xz2::bufread::XzDecoder;
use zstd::stream::read::Decoder as ZstdDecoder;

use upac_decoder_kit::ObjectStore;

use upac_types::CancelToken;
use upac_types::decoder::DecodeError;

use super::layout::entries::{
    INSTALL as INSTALL_ENTRY, PREFIX as ENTRY_PREFIX, PROPS as PROPS_ENTRY, REMOVE as REMOVE_ENTRY,
    SKIPPED as SKIPPED_ENTRIES,
};

const ZSTD_MAGIC: [u8; 4] = [0x28, 0xB5, 0x2F, 0xFD];
const XZ_MAGIC: [u8; 6] = [0xFD, b'7', b'z', b'X', b'Z', 0x00];
const GZIP_MAGIC: [u8; 2] = [0x1F, 0x8B];

pub struct XbpsArchive {
    pub props: String,
    pub install_present: bool,
    pub remove_present: bool,
    pub dumpfile: String,
}

impl XbpsArchive {
    pub fn read(package_path: &Path, objects: &ObjectStore, cancel: &CancelToken) -> Result<Self, DecodeError> {
        fn open_reader(file: File) -> Result<Box<dyn Read>, DecodeError> {
            let mut reader = BufReader::new(file);
            let magic = reader.fill_buf()?;

            if magic.starts_with(&ZSTD_MAGIC) {
                Ok(Box::new(ZstdDecoder::with_buffer(reader)?))
            } else if magic.starts_with(&XZ_MAGIC) {
                Ok(Box::new(XzDecoder::new(reader)))
            } else if magic.starts_with(&GZIP_MAGIC) {
                Ok(Box::new(GzDecoder::new(reader)))
            } else {
                Ok(Box::new(reader))
            }
        }

        let mut archive = Archive::new(open_reader(File::open(package_path)?)?);
        let mut dumpfile = String::new();

        let mut props = None;
        let mut install_present = false;
        let mut remove_present = false;

        for entry in archive.entries()? {
            if cancel.is_cancelled() {
                return Err(DecodeError::Cancelled);
            }

            let mut entry = entry?;
            let entry_path = entry.path()?.to_string_lossy().into_owned();

            match entry_path.strip_prefix(ENTRY_PREFIX).unwrap_or(&entry_path) {
                PROPS_ENTRY => props = Some(read_to_string(&mut entry)?),
                INSTALL_ENTRY => install_present = true,
                REMOVE_ENTRY => remove_present = true,
                skipped if SKIPPED_ENTRIES.contains(&skipped) => {}
                _ => objects.write_entry(&mut dumpfile, &mut entry)?,
            }
        }

        let props = props.ok_or(DecodeError::MissingMetadata)?;

        Ok(Self {
            props,
            install_present,
            remove_present,
            dumpfile,
        })
    }
}
