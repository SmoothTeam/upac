// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::fs::{File, metadata};
use std::io::{Cursor, Read};
use std::path::{Path, PathBuf};

use anyhow::Error as AnyhowError;

use flate2::read::GzDecoder;
use tar::Archive;
use tempfile::TempDir;
use xz2::read::XzDecoder;
use zip::ZipArchive;
use zstd::stream::read::Decoder as ZstdDecoder;

use upac::errors::CommonError;
use upac::orchestrator::Context;
use upac::orchestrator::stage::{NoRollback, RollbackGuard, Stage, StageResult};

use upac_abi::hook::{CancelToken, ProgressEventBuilder};

use crate::error::SetupError;
use crate::types::{GenesisInput, ResolvedSourceDir};

#[cfg(test)]
#[path = "../../tests/inline/source.rs"]
mod tests;

const ZIP_MAGIC: [u8; 4] = [0x50, 0x4B, 0x03, 0x04];
const SEVENZIP_MAGIC: [u8; 6] = [0x37, 0x7A, 0xBC, 0xAF, 0x27, 0x1C];
const ZSTD_MAGIC: [u8; 4] = [0x28, 0xB5, 0x2F, 0xFD];
const XZ_MAGIC: [u8; 6] = [0xFD, b'7', b'z', b'X', b'Z', 0x00];
const GZIP_MAGIC: [u8; 2] = [0x1F, 0x8B];

enum SourceArchive {
    Zip(File),
    SevenZip(PathBuf),
    Tar(Box<dyn Read>),
}

impl SourceArchive {
    fn sniff(path: &Path) -> Result<Self, SetupError> {
        let mut file = File::open(path)?;
        let mut magic = [0u8; 6];
        let bytes_read = file.read(&mut magic)?;
        let sniffed = &magic[..bytes_read];

        Ok(match () {
            _ if sniffed.starts_with(&ZIP_MAGIC) => SourceArchive::Zip(File::open(path)?),
            _ if sniffed.starts_with(&SEVENZIP_MAGIC) => SourceArchive::SevenZip(path.to_path_buf()),
            _ if sniffed.starts_with(&ZSTD_MAGIC) => Self::zstd_tar(sniffed, file)?,
            _ if sniffed.starts_with(&XZ_MAGIC) => Self::xz_tar(sniffed, file),
            _ if sniffed.starts_with(&GZIP_MAGIC) => Self::gzip_tar(sniffed, file),
            _ => Self::plain_tar(sniffed, file),
        })
    }

    fn zstd_tar(sniffed: &[u8], file: File) -> Result<Self, SetupError> {
        let chained = Cursor::new(sniffed.to_vec()).chain(file);
        Ok(SourceArchive::Tar(Box::new(ZstdDecoder::new(chained)?)))
    }

    fn xz_tar(sniffed: &[u8], file: File) -> Self {
        let chained = Cursor::new(sniffed.to_vec()).chain(file);
        SourceArchive::Tar(Box::new(XzDecoder::new(chained)))
    }

    fn gzip_tar(sniffed: &[u8], file: File) -> Self {
        let chained = Cursor::new(sniffed.to_vec()).chain(file);
        SourceArchive::Tar(Box::new(GzDecoder::new(chained)))
    }

    fn plain_tar(sniffed: &[u8], file: File) -> Self {
        SourceArchive::Tar(Box::new(Cursor::new(sniffed.to_vec()).chain(file)))
    }

    fn extract(self, destination: &Path) -> Result<(), SetupError> {
        match self {
            SourceArchive::Zip(file) => Self::extract_zip(file, destination),
            SourceArchive::SevenZip(path) => Self::extract_sevenzip(&path, destination),
            SourceArchive::Tar(reader) => Self::extract_tar(reader, destination),
        }
    }

    fn extract_zip(file: File, destination: &Path) -> Result<(), SetupError> {
        let mut archive = ZipArchive::new(file).map_err(AnyhowError::new)?;
        archive.extract(destination).map_err(AnyhowError::new)?;
        Ok(())
    }

    fn extract_sevenzip(path: &Path, destination: &Path) -> Result<(), SetupError> {
        sevenz_rust2::decompress_file(path, destination).map_err(AnyhowError::new)?;
        Ok(())
    }

    fn extract_tar(reader: Box<dyn Read>, destination: &Path) -> Result<(), SetupError> {
        Archive::new(reader).unpack(destination)?;
        Ok(())
    }
}

pub struct PrepareSourceStage;

impl Stage<SetupError> for PrepareSourceStage {
    fn run(
        &self, context: &mut Context, _cancel: &CancelToken, progress: ProgressEventBuilder,
    ) -> Result<(ProgressEventBuilder, StageResult, Box<dyn RollbackGuard>), SetupError> {
        let input = context.get::<GenesisInput>().ok_or(CommonError::MissingResult)?;
        let source_path = Path::new(&input.source);

        if metadata(source_path)?.is_dir() {
            context.put(ResolvedSourceDir(source_path.to_path_buf()));
            return Ok((progress, StageResult::Advance, Box::new(NoRollback)));
        }

        let archive = SourceArchive::sniff(source_path)?;
        let scratch = TempDir::new()?;
        archive.extract(scratch.path())?;

        context.put(ResolvedSourceDir(scratch.path().to_path_buf()));
        context.put(scratch);

        Ok((progress, StageResult::Advance, Box::new(NoRollback)))
    }
}
