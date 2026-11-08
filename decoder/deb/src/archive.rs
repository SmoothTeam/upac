// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::fs::File;
use std::io::{Read, read_to_string};
use std::path::Path;
use std::str::from_utf8;

use ar::Archive as ArArchive;
use flate2::read::GzDecoder;
use tar::{Archive as TarArchive, Entry};
use xz2::read::XzDecoder;
use zstd::stream::read::Decoder as ZstdDecoder;

use upac_composefs::tree::builder::TreeBuilder;

use upac_decoder_kit::ObjectStore;

use upac_types::CancelToken;
use upac_types::decoder::DecodeError;

use super::layout::control::ENTRY as CONTROL_ENTRY;
use super::layout::copyright::{DIR_PREFIX, ENTRY_SUFFIX, LICENSE_PREFIX};
use super::layout::members::{CONTROL_PREFIX, DATA_PREFIX};
use super::layout::scripts::{POSTINST, POSTRM, PREINST, PRERM};

const SCRIPTS: [&str; 4] = [PREINST, POSTINST, PRERM, POSTRM];

pub struct DebArchive {
    pub control: String,
    pub scripts: Vec<String>,
    pub license: Option<String>,
    pub dumpfile: String,
}

impl DebArchive {
    pub fn read(package_path: &Path, objects: &ObjectStore, cancel: &CancelToken) -> Result<Self, DecodeError> {
        fn open_member<'member>(
            name: &str, member: impl Read + 'member,
        ) -> Result<Box<dyn Read + 'member>, DecodeError> {
            match name {
                _ if name.ends_with(".zst") => Ok(Box::new(ZstdDecoder::new(member)?)),
                _ if name.ends_with(".xz") => Ok(Box::new(XzDecoder::new(member))),
                _ if name.ends_with(".gz") => Ok(Box::new(GzDecoder::new(member))),
                CONTROL_PREFIX | DATA_PREFIX => Ok(Box::new(member)),
                _ => Err(DecodeError::UnsupportedFormat),
            }
        }

        fn entry_name<Reader: Read>(entry: &Entry<'_, Reader>) -> Result<String, DecodeError> {
            let entry_path = entry.path()?.to_string_lossy().into_owned();

            Ok(entry_path.strip_prefix("./").unwrap_or(&entry_path).to_owned())
        }

        fn parse_license(content: &[u8]) -> Option<String> {
            from_utf8(content)
                .ok()?
                .lines()
                .map(str::trim)
                .find_map(|line| line.strip_prefix(LICENSE_PREFIX))
                .map(|value| value.trim().to_owned())
                .filter(|value| !value.is_empty())
        }

        let mut outer = ArArchive::new(File::open(package_path)?);
        let mut builder = TreeBuilder::new();

        let mut control = None;
        let mut scripts = Vec::new();
        let mut license = None;
        let mut is_data_read = false;

        while let Some(member) = outer.next_entry() {
            if cancel.is_cancelled() {
                return Err(DecodeError::Cancelled);
            }

            let member = member?;
            let name =
                String::from_utf8(member.header().identifier().to_vec()).map_err(|_| DecodeError::InvalidUtf8)?;

            if name.starts_with(CONTROL_PREFIX) && control.is_none() {
                let mut archive = TarArchive::new(open_member(&name, member)?);

                for entry in archive.entries()? {
                    if cancel.is_cancelled() {
                        return Err(DecodeError::Cancelled);
                    }

                    let mut entry = entry?;
                    let entry_name = entry_name(&entry)?;

                    if entry_name == CONTROL_ENTRY {
                        control = Some(read_to_string(&mut entry)?);
                    } else if SCRIPTS.contains(&entry_name.as_str()) {
                        scripts.push(entry_name);
                    }
                }
            } else if name.starts_with(DATA_PREFIX) && !is_data_read {
                is_data_read = true;

                let mut archive = TarArchive::new(open_member(&name, member)?);

                for entry in archive.entries()? {
                    if cancel.is_cancelled() {
                        return Err(DecodeError::Cancelled);
                    }

                    let mut entry = entry?;
                    let entry_name = entry_name(&entry)?;

                    if license.is_none() && entry_name.starts_with(DIR_PREFIX) && entry_name.ends_with(ENTRY_SUFFIX) {
                        let mut content = Vec::new();
                        entry.read_to_end(&mut content)?;

                        license = parse_license(&content);
                        objects.insert_entry_with(&mut builder, &mut entry, &content)?;
                    } else {
                        objects.insert_entry(&mut builder, &mut entry)?;
                    }
                }
            }
        }

        let control = control.ok_or(DecodeError::MissingMetadata)?;

        if !is_data_read {
            return Err(DecodeError::MissingMetadata);
        }

        let dumpfile = builder
            .view()
            .to_dumpfile()
            .map_err(|_| DecodeError::MalformedMetadata)?;

        Ok(Self {
            control,
            scripts,
            license,
            dumpfile,
        })
    }
}
