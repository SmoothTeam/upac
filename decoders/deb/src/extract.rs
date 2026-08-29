// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::fs::{self, File};
use std::io::{Cursor, Read};
use std::path::Path;

use ar::Archive as ArArchive;
use flate2::read::GzDecoder;
use tar::Archive as TarArchive;
use xz2::read::XzDecoder;
use zstd::stream::read::Decoder as ZstdDecoder;

use upac_abi::hook::CancelToken;

use crate::deb::{
    CONTROL_ENTRY, CONTROL_TAR_PREFIX, COPYRIGHT_DIR_PREFIX, COPYRIGHT_ENTRY_SUFFIX, DATA_TAR_PREFIX, POSTINST_FILE,
    POSTRM_FILE, PREINST_FILE, PRERM_FILE,
};
use crate::error::DecodeError;

const SCRIPT_FILES: [&str; 4] = [PREINST_FILE, POSTINST_FILE, PRERM_FILE, POSTRM_FILE];

pub struct ExtractedMetadata {
    pub control: String,
    pub scripts_present: Vec<String>,
    pub license: Option<String>,
}

pub fn extract(package_path: &str, output_dir: &str, cancel: &CancelToken) -> Result<ExtractedMetadata, DecodeError> {
    let file = File::open(package_path)?;
    let mut outer = ArArchive::new(file);

    let mut control_tar: Option<(String, Vec<u8>)> = None;
    let mut data_tar: Option<(String, Vec<u8>)> = None;

    while let Some(entry) = outer.next_entry() {
        if cancel.is_cancelled() {
            return Err(DecodeError::Cancelled);
        }

        let mut entry = entry?;
        let name = String::from_utf8(entry.header().identifier().to_vec()).map_err(|_| DecodeError::InvalidUtf8)?;

        if name.starts_with(CONTROL_TAR_PREFIX) && control_tar.is_none() {
            let mut bytes = Vec::new();
            entry.read_to_end(&mut bytes)?;
            control_tar = Some((name, bytes));
        } else if name.starts_with(DATA_TAR_PREFIX) && data_tar.is_none() {
            let mut bytes = Vec::new();
            entry.read_to_end(&mut bytes)?;
            data_tar = Some((name, bytes));
        }
    }

    let (control_name, control_bytes) = control_tar.ok_or(DecodeError::MissingControl)?;
    let (data_name, data_bytes) = data_tar.ok_or(DecodeError::MissingControl)?;

    let (control, scripts_present) = extract_control(control_bytes, &control_name, cancel)?;
    let license = extract_data(data_bytes, &data_name, output_dir, cancel)?;

    Ok(ExtractedMetadata {
        control,
        scripts_present,
        license,
    })
}

fn extract_control(
    bytes: Vec<u8>, member_name: &str, cancel: &CancelToken,
) -> Result<(String, Vec<String>), DecodeError> {
    let reader = open_tar_reader(member_name, bytes)?;
    let mut archive = TarArchive::new(reader);

    let mut control = None;
    let mut scripts_present = Vec::new();

    for entry in archive.entries()? {
        if cancel.is_cancelled() {
            return Err(DecodeError::Cancelled);
        }

        let mut entry = entry?;
        let entry_path = entry.path()?.to_string_lossy().into_owned();
        let normalized = entry_path.strip_prefix("./").unwrap_or(&entry_path);

        if normalized == CONTROL_ENTRY {
            control = Some(read_entry_to_string(&mut entry)?);
        } else if SCRIPT_FILES.contains(&normalized) {
            scripts_present.push(normalized.to_owned());
        }
    }

    control
        .map(|control| (control, scripts_present))
        .ok_or(DecodeError::MissingControl)
}

fn extract_data(
    bytes: Vec<u8>, member_name: &str, output_dir: &str, cancel: &CancelToken,
) -> Result<Option<String>, DecodeError> {
    let reader = open_tar_reader(member_name, bytes)?;
    let mut archive = TarArchive::new(reader);

    let mut license = None;

    for entry in archive.entries()? {
        if cancel.is_cancelled() {
            return Err(DecodeError::Cancelled);
        }

        let mut entry = entry?;
        let entry_path = entry.path()?.to_string_lossy().into_owned();
        let normalized = entry_path.strip_prefix("./").unwrap_or(&entry_path);

        if license.is_none()
            && normalized.starts_with(COPYRIGHT_DIR_PREFIX)
            && normalized.ends_with(COPYRIGHT_ENTRY_SUFFIX)
        {
            let mut bytes = Vec::new();
            entry.read_to_end(&mut bytes)?;

            let target = Path::new(output_dir).join(normalized);
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(&target, &bytes)?;

            license = String::from_utf8(bytes)
                .ok()
                .and_then(|content| parse_license_from_copyright(&content));
            continue;
        }

        entry.unpack_in(output_dir)?;
    }

    Ok(license)
}

fn parse_license_from_copyright(content: &str) -> Option<String> {
    content
        .lines()
        .map(str::trim)
        .find_map(|line| line.strip_prefix("License:"))
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
}

fn read_entry_to_string<R: Read>(entry: &mut tar::Entry<'_, R>) -> Result<String, DecodeError> {
    let mut bytes = Vec::new();
    entry.read_to_end(&mut bytes)?;

    String::from_utf8(bytes).map_err(|_| DecodeError::InvalidUtf8)
}

fn open_tar_reader(member_name: &str, bytes: Vec<u8>) -> Result<Box<dyn Read>, DecodeError> {
    let cursor = Cursor::new(bytes);

    if member_name.ends_with(".zst") {
        Ok(Box::new(ZstdDecoder::new(cursor)?))
    } else if member_name.ends_with(".xz") {
        Ok(Box::new(XzDecoder::new(cursor)))
    } else if member_name.ends_with(".gz") {
        Ok(Box::new(GzDecoder::new(cursor)))
    } else if member_name == CONTROL_TAR_PREFIX || member_name == DATA_TAR_PREFIX {
        Ok(Box::new(cursor))
    } else {
        Err(DecodeError::UnsupportedFormat)
    }
}
