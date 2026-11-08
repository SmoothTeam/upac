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
use composefs::tree::{Leaf, RegularFile};

use cpio::newc::Reader as CpioReader;
use flate2::read::GzDecoder;
use xz2::read::XzDecoder;
use zstd::stream::read::Decoder as ZstdDecoder;

use nix::sys::stat::{Mode, SFlag};

use upac_abi::{OWNER_SEPARATOR, OWNER_XATTR};

use upac_composefs::tree::entry::TreeEntry;

use upac_decoder_kit::{ObjectStore, write_tree_entry};

use upac_types::CancelToken;
use upac_types::decoder::DecodeError;

use super::capabilities::{CAPABILITY_XATTR, FileCapabilities};
use super::header::Header;
use super::layout::owners::ROOT;
use super::layout::payload::{DEFAULT_COMPRESSOR, FORMAT as CPIO_FORMAT, GZIP, NONE, XZ, ZSTD};
use super::layout::tags::{
    BASE_NAMES, DIR_INDEXES, DIR_NAMES, FILE_CAPS, FILE_GROUP_NAME, FILE_USER_NAME, PAYLOAD_COMPRESSOR, PAYLOAD_FORMAT,
};

const DIRECTORY: u32 = SFlag::S_IFDIR.bits();
const REGULAR: u32 = SFlag::S_IFREG.bits();
const SYMLINK: u32 = SFlag::S_IFLNK.bits();

pub struct RpmArchive {
    pub header: Header,
    pub dumpfile: String,
}

impl RpmArchive {
    pub fn read(package_path: &Path, objects: &ObjectStore, cancel: &CancelToken) -> Result<Self, DecodeError> {
        fn strip_root(path: &Path) -> PathBuf {
            path.components()
                .filter(|component| !matches!(component, Component::CurDir | Component::RootDir))
                .collect()
        }

        fn read_xattrs(header: &Header) -> Result<HashMap<PathBuf, BTreeMap<Box<OsStr>, Box<[u8]>>>, DecodeError> {
            let users = header.string_array(FILE_USER_NAME)?;
            let groups = header.string_array(FILE_GROUP_NAME)?;
            let capabilities = header.string_array(FILE_CAPS)?;
            let base_names = header.string_array(BASE_NAMES)?;
            let dir_names = header.string_array(DIR_NAMES)?;
            let dir_indexes = header.int32_array(DIR_INDEXES)?;

            let mut path_xattrs = HashMap::new();

            for (index, base_name) in base_names.iter().enumerate() {
                let mut xattrs = BTreeMap::new();

                if let (Some(user), Some(group)) = (users.get(index), groups.get(index))
                    && !(user == ROOT && group == ROOT)
                {
                    xattrs.insert(
                        Box::from(OsStr::new(OWNER_XATTR)),
                        Box::from(format!("{user}{OWNER_SEPARATOR}{group}").as_bytes()),
                    );
                }

                if let Some(text) = capabilities.get(index)
                    && !text.is_empty()
                {
                    xattrs.insert(
                        Box::from(OsStr::from_bytes(CAPABILITY_XATTR)),
                        FileCapabilities::parse(text)?.to_xattr(),
                    );
                }

                if xattrs.is_empty() {
                    continue;
                }

                let dir_name = dir_indexes
                    .get(index)
                    .and_then(|dir_index| usize::try_from(*dir_index).ok())
                    .and_then(|dir_index| dir_names.get(dir_index))
                    .ok_or(DecodeError::MalformedMetadata)?;

                path_xattrs.insert(strip_root(&Path::new(dir_name).join(base_name)), xattrs);
            }

            Ok(path_xattrs)
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

        let mut path_xattrs = read_xattrs(&header)?;

        let mut dumpfile = String::new();
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

            let path = strip_root(Path::new(entry.name()));
            if path.as_os_str().is_empty() {
                payload = entry_reader.finish()?;
                continue;
            }

            let stat = Stat {
                st_mode: Mode::from_bits_truncate(entry.mode()).bits(),
                st_uid: entry.uid(),
                st_gid: entry.gid(),
                st_mtim_sec: i64::from(entry.mtime()),
                st_mtim_nsec: 0,
                xattrs: path_xattrs.remove(&path).unwrap_or_default(),
            };

            match entry.mode() & SFlag::S_IFMT.bits() {
                DIRECTORY => write_tree_entry(&mut dumpfile, TreeEntry::Directory(path, stat))?,
                SYMLINK => {
                    let mut target = Vec::new();
                    entry_reader.read_to_end(&mut target)?;

                    let content = LeafContent::Symlink(OsStr::from_bytes(&target).into());

                    write_tree_entry(&mut dumpfile, TreeEntry::Leaf(path, Leaf { stat, content }))?;
                }
                REGULAR if entry.nlink() > 1 && entry.file_size() == 0 => {
                    pending_links.entry(entry.ino()).or_default().push((path, stat));
                }
                REGULAR => {
                    let content = LeafContent::Regular(objects.store(&mut entry_reader, u64::from(entry.file_size()))?);

                    write_tree_entry(&mut dumpfile, TreeEntry::Leaf(path.clone(), Leaf { stat, content }))?;

                    for (link, _) in pending_links.remove(&entry.ino()).unwrap_or_default() {
                        write_tree_entry(&mut dumpfile, TreeEntry::HardLink(link, path.clone()))?;
                    }
                }
                _ => {}
            }

            payload = entry_reader.finish()?;
        }

        for (_, links) in pending_links {
            let mut links = links.into_iter();

            let Some((original, stat)) = links.next() else {
                continue;
            };

            let content = LeafContent::Regular(RegularFile::Inline(Box::default()));

            write_tree_entry(&mut dumpfile, TreeEntry::Leaf(original.clone(), Leaf { stat, content }))?;

            for (link, _) in links {
                write_tree_entry(&mut dumpfile, TreeEntry::HardLink(link, original.clone()))?;
            }
        }

        Ok(Self { header, dumpfile })
    }
}
