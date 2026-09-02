// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::fs::{File, read_dir, read_link, read_to_string};
use std::io::Read;
use std::path::Path;

use sha2::{Digest, Sha256};

use toml::from_str;

use upac_types::PackageMeta;

use crate::error::SetupError;
use crate::layout::meta::FILENAME;

pub struct SourceDir<'src> {
    pub path: &'src Path,
}

impl SourceDir<'_> {
    pub fn read(&self, filename: Option<&str>) -> Result<PackageMeta, SetupError> {
        let content = read_to_string(self.path.join(filename.unwrap_or(FILENAME)))?;

        Ok(from_str(&content)?)
    }

    pub fn checksum(&self, include_config: bool) -> Result<([u8; 32], u64), SetupError> {
        let mut accumulator = Accumulator {
            hasher: Sha256::new(),
            installed_size: 0,
        };

        let sections: &[&str] = if include_config { &["usr", "etc"] } else { &["usr"] };

        for &section in sections {
            let section_dir = self.path.join(section);
            if section_dir.is_dir() {
                accumulator.hasher.update(section.as_bytes());
                accumulator.hash_dir(&section_dir)?;
            }
        }

        Ok((accumulator.hasher.finalize().into(), accumulator.installed_size))
    }
}

struct Accumulator {
    hasher: Sha256,
    installed_size: u64,
}

impl Accumulator {
    fn hash_dir(&mut self, dir: &Path) -> Result<(), SetupError> {
        let mut entries = read_dir(dir)?.collect::<Result<Vec<_>, _>>()?;
        entries.sort_by_key(|entry| entry.file_name());

        for entry in entries {
            let metadata = entry.metadata()?;
            self.hasher.update(entry.file_name().as_encoded_bytes());

            if metadata.is_dir() {
                self.hash_dir(&entry.path())?;
            } else if metadata.is_symlink() {
                self.hasher
                    .update(read_link(entry.path())?.as_os_str().as_encoded_bytes());
            } else {
                let mut file = File::open(entry.path())?;
                let mut buffer = [0u8; 65536];

                loop {
                    let bytes_read = file.read(&mut buffer)?;
                    if bytes_read == 0 {
                        break;
                    }
                    self.hasher.update(&buffer[..bytes_read]);
                }

                self.installed_size += metadata.len();
            }
        }

        Ok(())
    }
}
