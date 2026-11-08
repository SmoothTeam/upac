// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::collections::{BTreeMap, HashMap};
use std::ffi::OsStr;
use std::fs::File;
use std::io::Read;
use std::os::unix::ffi::OsStrExt;
use std::path::{Component, Path, PathBuf};

use composefs::generic_tree::{LeafContent, Stat};
use composefs::tree::RegularFile;

use cpio::newc::Reader as CpioReader;
use flate2::read::GzDecoder;
use xz2::read::XzDecoder;
use zstd::stream::read::Decoder as ZstdDecoder;

use upac_abi::{OWNER_SEPARATOR, OWNER_XATTR};

use upac_composefs::error::RepoError;
use upac_composefs::tree::builder::TreeBuilder;

use upac_decoder_kit::ObjectStore;

use upac_types::CancelToken;
use upac_types::decoder::DecodeError;

use super::header::Header;
use super::layout::mode::{DIRECTORY, MISSING_DIRECTORY, PERMISSION_MASK, REGULAR, SYMLINK, TYPE_MASK};
use super::layout::owners::ROOT;
use super::layout::payload::{DEFAULT_COMPRESSOR, FORMAT as CPIO_FORMAT, GZIP, NONE, XZ, ZSTD};
use super::layout::tags::{
    BASE_NAMES, DIR_INDEXES, DIR_NAMES, FILE_GROUP_NAME, FILE_USER_NAME, PAYLOAD_COMPRESSOR, PAYLOAD_FORMAT,
};

pub struct RpmArchive {
    pub header: Header,
    pub dumpfile: String,
}

impl RpmArchive {
    pub fn read(package_path: &Path, objects: &ObjectStore, cancel: &CancelToken) -> Result<Self, DecodeError> {
        fn normalize(path: &Path) -> Result<PathBuf, DecodeError> {
            if path
                .components()
                .any(|component| matches!(component, Component::ParentDir))
            {
                return Err(DecodeError::MalformedMetadata);
            }

            Ok(path
                .components()
                .filter(|component| matches!(component, Component::Normal(_)))
                .collect())
        }

        fn read_owners(header: &Header) -> Result<HashMap<PathBuf, String>, DecodeError> {
            let users = header.string_array(FILE_USER_NAME)?;
            let groups = header.string_array(FILE_GROUP_NAME)?;
            let base_names = header.string_array(BASE_NAMES)?;
            let dir_names = header.string_array(DIR_NAMES)?;
            let dir_indexes = header.int32_array(DIR_INDEXES)?;

            let mut owners = HashMap::new();

            for (index, base_name) in base_names.iter().enumerate() {
                let (Some(user), Some(group)) = (users.get(index), groups.get(index)) else {
                    continue;
                };

                if user == ROOT && group == ROOT {
                    continue;
                }

                let dir_name = dir_indexes
                    .get(index)
                    .and_then(|dir_index| usize::try_from(*dir_index).ok())
                    .and_then(|dir_index| dir_names.get(dir_index))
                    .ok_or(DecodeError::MalformedMetadata)?;

                owners.insert(
                    normalize(&Path::new(dir_name).join(base_name))?,
                    format!("{user}{OWNER_SEPARATOR}{group}"),
                );
            }

            Ok(owners)
        }

        fn insert_parents(builder: &mut TreeBuilder, path: &Path) -> Result<(), DecodeError> {
            let mut missing: Vec<&Path> = path
                .ancestors()
                .skip(1)
                .filter(|ancestor| !ancestor.as_os_str().is_empty())
                .take_while(|ancestor| !builder.view().check(ancestor))
                .collect();
            missing.reverse();

            for directory in missing {
                builder
                    .insert_dir(
                        directory,
                        Stat {
                            st_mode: MISSING_DIRECTORY,
                            ..Stat::uninitialized()
                        },
                    )
                    .map_err(|_| DecodeError::MalformedMetadata)?;
            }

            Ok(())
        }

        let mut file = File::open(package_path)?;
        let header = Header::read(&mut file)?;

        if header.string(PAYLOAD_FORMAT)?.as_deref().unwrap_or(CPIO_FORMAT) != CPIO_FORMAT {
            return Err(DecodeError::UnsupportedFormat);
        }

        let compressor = header
            .string(PAYLOAD_COMPRESSOR)?
            .unwrap_or_else(|| DEFAULT_COMPRESSOR.to_owned());

        let mut payload: Box<dyn Read> = match compressor.as_str() {
            GZIP => Box::new(GzDecoder::new(file)),
            XZ => Box::new(XzDecoder::new(file)),
            ZSTD => Box::new(ZstdDecoder::new(file)?),
            NONE => Box::new(file),
            _ => return Err(DecodeError::UnsupportedFormat),
        };

        let owners = read_owners(&header)?;

        let mut builder = TreeBuilder::new();
        let mut pending_links: HashMap<u32, Vec<(PathBuf, Stat)>> = HashMap::new();

        loop {
            if cancel.is_cancelled() {
                return Err(DecodeError::Cancelled);
            }

            let mut entry_reader = CpioReader::new(payload)?;
            let entry = entry_reader.entry().clone();

            if entry.is_trailer() {
                break;
            }

            let path = normalize(Path::new(entry.name()))?;
            if path.as_os_str().is_empty() {
                payload = entry_reader.finish()?;
                continue;
            }

            insert_parents(&mut builder, &path)?;

            let mut xattrs = BTreeMap::new();
            if let Some(owner) = owners.get(&path) {
                xattrs.insert(Box::from(OsStr::new(OWNER_XATTR)), Box::from(owner.as_bytes()));
            }

            let stat = Stat {
                st_mode: entry.mode() & PERMISSION_MASK,
                st_uid: entry.uid(),
                st_gid: entry.gid(),
                st_mtim_sec: i64::from(entry.mtime()),
                st_mtim_nsec: 0,
                xattrs,
            };

            let inserted = match entry.mode() & TYPE_MASK {
                DIRECTORY => match builder.try_insert_dir(&path, stat.clone()) {
                    Err(RepoError::AlreadyExists) => builder.update_metadata(&path, stat),
                    inserted => inserted,
                },
                SYMLINK => {
                    let mut target = Vec::new();
                    entry_reader.read_to_end(&mut target)?;

                    builder.insert(&path, LeafContent::Symlink(OsStr::from_bytes(&target).into()), stat)
                }
                REGULAR if entry.nlink() > 1 && entry.file_size() == 0 => {
                    pending_links.entry(entry.ino()).or_default().push((path, stat));

                    Ok(())
                }
                REGULAR => {
                    let content = objects.store(&mut entry_reader, u64::from(entry.file_size()))?;

                    builder
                        .insert(&path, LeafContent::Regular(content), stat)
                        .and_then(|()| {
                            pending_links
                                .remove(&entry.ino())
                                .unwrap_or_default()
                                .iter()
                                .try_for_each(|(link, _)| builder.insert_hard_link(link, &path))
                        })
                }
                _ => Ok(()),
            };

            inserted.map_err(|_| DecodeError::MalformedMetadata)?;

            payload = entry_reader.finish()?;
        }

        for (_, links) in pending_links {
            let mut links = links.into_iter();

            let Some((original, stat)) = links.next() else {
                continue;
            };

            builder
                .insert(
                    &original,
                    LeafContent::Regular(RegularFile::Inline(Box::default())),
                    stat,
                )
                .and_then(|()| links.try_for_each(|(link, _)| builder.insert_hard_link(&link, &original)))
                .map_err(|_| DecodeError::MalformedMetadata)?;
        }

        let dumpfile = builder
            .view()
            .to_dumpfile()
            .map_err(|_| DecodeError::MalformedMetadata)?;

        Ok(Self { header, dumpfile })
    }
}
