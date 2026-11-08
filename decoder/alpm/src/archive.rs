// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::ffi::OsStr;
use std::fs::File;
use std::io::{Read, read_to_string};
use std::path::Path;

use flate2::read::GzDecoder;
use tar::Archive;
use xz2::read::XzDecoder;
use zstd::stream::read::Decoder as ZstdDecoder;

use upac_composefs::tree::builder::TreeBuilder;

use upac_decoder_kit::ObjectStore;

use upac_types::CancelToken;
use upac_types::decoder::DecodeError;

use super::layout::entries::{INSTALL as INSTALL_ENTRY, PKGINFO as PKGINFO_ENTRY, SKIPPED as SKIPPED_ENTRIES};

pub struct AlpmArchive {
    pub pkginfo: String,
    pub install: Option<String>,
    pub dumpfile: String,
}

impl AlpmArchive {
    pub fn read(package_path: &Path, objects: &ObjectStore, cancel: &CancelToken) -> Result<Self, DecodeError> {
        fn open_reader(package_path: &Path, file: File) -> Result<Box<dyn Read>, DecodeError> {
            let file_name = package_path
                .file_name()
                .and_then(OsStr::to_str)
                .ok_or(DecodeError::UnsupportedFormat)?;

            let Some((_, compression)) = file_name.rsplit_once(".pkg.tar") else {
                return Err(DecodeError::UnsupportedFormat);
            };

            match compression {
                ".zst" => Ok(Box::new(ZstdDecoder::new(file)?)),
                ".xz" => Ok(Box::new(XzDecoder::new(file))),
                ".gz" => Ok(Box::new(GzDecoder::new(file))),
                "" => Ok(Box::new(file)),
                _ => Err(DecodeError::UnsupportedFormat),
            }
        }

        let mut archive = Archive::new(open_reader(package_path, File::open(package_path)?)?);
        let mut builder = TreeBuilder::new();

        let mut pkginfo = None;
        let mut install = None;

        for entry in archive.entries()? {
            if cancel.is_cancelled() {
                return Err(DecodeError::Cancelled);
            }

            let mut entry = entry?;
            let entry_path = entry.path()?.to_string_lossy().into_owned();

            match entry_path.as_str() {
                PKGINFO_ENTRY => pkginfo = Some(read_to_string(&mut entry)?),
                INSTALL_ENTRY => install = Some(read_to_string(&mut entry)?),
                skipped if SKIPPED_ENTRIES.contains(&skipped) => {}
                _ => objects.insert_entry(&mut builder, &mut entry)?,
            }
        }

        let pkginfo = pkginfo.ok_or(DecodeError::MissingMetadata)?;
        let dumpfile = builder
            .view()
            .to_dumpfile()
            .map_err(|_| DecodeError::MalformedMetadata)?;

        Ok(Self {
            pkginfo,
            install,
            dumpfile,
        })
    }
}
