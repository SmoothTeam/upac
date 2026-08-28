// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::fs::File;
use std::io::Read;

use flate2::read::GzDecoder;
use tar::Archive;
use xz2::read::XzDecoder;
use zstd::stream::read::Decoder as ZstdDecoder;

use upac_abi::hook::CancelToken;

use crate::alpm::{BUILDINFO_ENTRY, CHANGELOG_ENTRY, INSTALL_ENTRY, MTREE_ENTRY, PKGINFO_ENTRY};
use crate::error::DecodeError;

const JUNK_ENTRIES: [&str; 3] = [BUILDINFO_ENTRY, MTREE_ENTRY, CHANGELOG_ENTRY];

pub struct ExtractedMetadata {
    pub pkginfo: String,
    pub install: Option<String>,
}

pub fn extract(package_path: &str, output_dir: &str, cancel: &CancelToken) -> Result<ExtractedMetadata, DecodeError> {
    let file = File::open(package_path)?;
    let reader = open_reader(package_path, file)?;

    let mut archive = Archive::new(reader);

    let mut pkginfo = None;
    let mut install = None;

    for entry in archive.entries()? {
        if cancel.is_cancelled() {
            return Err(DecodeError::Cancelled);
        }

        let mut entry = entry?;
        let entry_path = entry.path()?.to_string_lossy().into_owned();

        if entry_path == PKGINFO_ENTRY {
            pkginfo = Some(read_entry_to_string(&mut entry)?);
            continue;
        }

        if entry_path == INSTALL_ENTRY {
            install = Some(read_entry_to_string(&mut entry)?);
            continue;
        }

        if JUNK_ENTRIES.contains(&entry_path.as_str()) {
            continue;
        }

        entry.unpack_in(output_dir)?;
    }

    pkginfo
        .map(|pkginfo| ExtractedMetadata { pkginfo, install })
        .ok_or(DecodeError::MissingPkgInfo)
}

fn read_entry_to_string<R: Read>(entry: &mut tar::Entry<'_, R>) -> Result<String, DecodeError> {
    let mut bytes = Vec::new();
    entry.read_to_end(&mut bytes)?;

    String::from_utf8(bytes).map_err(|_| DecodeError::InvalidUtf8)
}

fn open_reader(package_path: &str, file: File) -> Result<Box<dyn Read>, DecodeError> {
    if package_path.ends_with(".pkg.tar.zst") {
        Ok(Box::new(ZstdDecoder::new(file)?))
    } else if package_path.ends_with(".pkg.tar.xz") {
        Ok(Box::new(XzDecoder::new(file)))
    } else if package_path.ends_with(".pkg.tar.gz") {
        Ok(Box::new(GzDecoder::new(file)))
    } else if package_path.ends_with(".pkg.tar") {
        Ok(Box::new(file))
    } else {
        Err(DecodeError::UnsupportedFormat)
    }
}
