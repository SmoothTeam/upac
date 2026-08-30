// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::fs::{self, File};
use std::io::Read;
use std::os::unix::fs::symlink;
use std::path::{Component, Path};

use cpio::newc::Reader as CpioReader;
use flate2::read::GzDecoder;
use xz2::read::XzDecoder;
use zstd::stream::read::Decoder as ZstdDecoder;

use upac_abi::decoder::DecodeError;
use upac_abi::hook::CancelToken;

use crate::header::Header;
use crate::rpm::{PAYLOAD_COMPRESSOR_TAG, PAYLOAD_FORMAT_TAG};

const MODE_TYPE_MASK: u32 = 0o170000;
const MODE_TYPE_DIRECTORY: u32 = 0o040000;
const MODE_TYPE_REGULAR: u32 = 0o100000;
const MODE_TYPE_SYMLINK: u32 = 0o120000;

pub fn extract(file: File, header: &Header, output_dir: &str, cancel: &CancelToken) -> Result<(), DecodeError> {
    let format = header.string(PAYLOAD_FORMAT_TAG)?.unwrap_or_else(|| "cpio".to_owned());
    if format != "cpio" {
        return Err(DecodeError::UnsupportedFormat);
    }

    let compressor = header
        .string(PAYLOAD_COMPRESSOR_TAG)?
        .unwrap_or_else(|| "gzip".to_owned());

    let mut reader = open_decompressor(&compressor, file)?;

    loop {
        if cancel.is_cancelled() {
            return Err(DecodeError::Cancelled);
        }

        let mut entry_reader = CpioReader::new(reader)?;
        let entry = entry_reader.entry().clone();

        if entry.is_trailer() {
            break;
        }

        let relative_path = entry.name().trim_start_matches("./");
        if relative_path.is_empty() {
            reader = entry_reader.finish()?;
            continue;
        }

        if Path::new(relative_path)
            .components()
            .any(|component| matches!(component, Component::ParentDir))
        {
            return Err(DecodeError::MalformedMetadata);
        }

        let target_path = Path::new(output_dir).join(relative_path);

        reader = match entry.mode() & MODE_TYPE_MASK {
            MODE_TYPE_DIRECTORY => {
                fs::create_dir_all(&target_path)?;
                entry_reader.finish()?
            }
            MODE_TYPE_SYMLINK => {
                let mut link_target = Vec::new();
                entry_reader.read_to_end(&mut link_target)?;
                let link_target = String::from_utf8(link_target).map_err(|_| DecodeError::InvalidUtf8)?;

                if let Some(parent) = target_path.parent() {
                    fs::create_dir_all(parent)?;
                }
                symlink(link_target, &target_path)?;

                entry_reader.finish()?
            }
            MODE_TYPE_REGULAR => {
                if let Some(parent) = target_path.parent() {
                    fs::create_dir_all(parent)?;
                }

                let mut out = File::create(&target_path)?;
                entry_reader.to_writer(&mut out)?
            }
            _ => entry_reader.finish()?,
        };
    }

    Ok(())
}

fn open_decompressor(compressor: &str, file: File) -> Result<Box<dyn Read>, DecodeError> {
    match compressor {
        "gzip" => Ok(Box::new(GzDecoder::new(file))),
        "xz" => Ok(Box::new(XzDecoder::new(file))),
        "zstd" => Ok(Box::new(ZstdDecoder::new(file)?)),
        "none" => Ok(Box::new(file)),
        _ => Err(DecodeError::UnsupportedFormat),
    }
}
