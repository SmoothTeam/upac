// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later

use std::fs::{File, read_dir, read_link, read_to_string};
use std::io::Read;
use std::path::Path;

use sha2::{Digest, Sha256};

use toml::from_str;

use upac_types::PackageMeta;

use crate::error::SetupError;
use crate::layout::meta::FILENAME;

pub fn read(source_dir: &Path, filename: Option<&str>) -> Result<PackageMeta, SetupError> {
    let content = read_to_string(source_dir.join(filename.unwrap_or(FILENAME)))?;

    Ok(from_str(&content)?)
}

pub fn checksum(source_dir: &Path) -> Result<([u8; 32], u64), SetupError> {
    let mut hasher = Sha256::new();
    let mut installed_size = 0u64;

    for section in ["usr", "etc"] {
        let section_dir = source_dir.join(section);
        if section_dir.is_dir() {
            hasher.update(section.as_bytes());
            hash_dir(&section_dir, &mut hasher, &mut installed_size)?;
        }
    }

    Ok((hasher.finalize().into(), installed_size))
}

fn hash_dir(dir: &Path, hasher: &mut Sha256, installed_size: &mut u64) -> Result<(), SetupError> {
    let mut entries = read_dir(dir)?.collect::<Result<Vec<_>, _>>()?;
    entries.sort_by_key(|entry| entry.file_name());

    for entry in entries {
        let metadata = entry.metadata()?;
        hasher.update(entry.file_name().as_encoded_bytes());

        if metadata.is_dir() {
            hash_dir(&entry.path(), hasher, installed_size)?;
        } else if metadata.is_symlink() {
            hasher.update(read_link(entry.path())?.as_os_str().as_encoded_bytes());
        } else {
            let mut file = File::open(entry.path())?;
            let mut buffer = [0u8; 65536];

            loop {
                let bytes_read = file.read(&mut buffer)?;
                if bytes_read == 0 {
                    break;
                }
                hasher.update(&buffer[..bytes_read]);
            }

            *installed_size += metadata.len();
        }
    }

    Ok(())
}
