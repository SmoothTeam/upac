// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::iter::once;
use std::os::unix::ffi::OsStrExt;
use std::path::{Component, Path, PathBuf};

use composefs::dumpfile::{write_directory, write_hardlink, write_leaf};
use composefs::dumpfile_parse::{Entry, Item};
use composefs::fsverity::FsVerityHashValue;
use composefs::generic_tree::{LeafContent, Stat};
use composefs::tree::{Leaf, RegularFile};

use nix::sys::stat::{Mode, SFlag};

use crate::error::RepoError;
use crate::{ObjectID, Repo};

pub enum TreeEntry {
    Directory(PathBuf, Stat),
    Leaf(PathBuf, Leaf<ObjectID>),
    HardLink(PathBuf, PathBuf),
}

impl TreeEntry {
    pub fn parse(line: &str, repo: &Repo) -> Result<Self, RepoError> {
        fn strip_root(path: &Path) -> Result<PathBuf, RepoError> {
            let relative = path
                .strip_prefix(Component::RootDir)
                .map_err(|_| RepoError::InvalidDumpfile)?;

            if relative.as_os_str().is_empty() {
                return Err(RepoError::InvalidDumpfile);
            }

            Ok(relative.to_path_buf())
        }

        fn read_stat(entry: &Entry<'_>) -> Result<Stat, RepoError> {
            Ok(Stat {
                st_mode: Mode::from_bits_truncate(entry.mode).bits(),
                st_uid: entry.uid,
                st_gid: entry.gid,
                st_mtim_sec: i64::try_from(entry.mtime.sec).map_err(|_| RepoError::InvalidDumpfile)?,
                st_mtim_nsec: u32::try_from(entry.mtime.nsec).map_err(|_| RepoError::InvalidDumpfile)?,
                xattrs: entry
                    .xattrs
                    .iter()
                    .map(|xattr| (Box::from(&*xattr.key), Box::from(&*xattr.value)))
                    .collect(),
            })
        }

        let entry = Entry::parse(line).map_err(|_| RepoError::InvalidDumpfile)?;
        let path = strip_root(&entry.path)?;

        let content = match &entry.item {
            Item::Directory { .. } => return Ok(Self::Directory(path, read_stat(&entry)?)),
            Item::Hardlink { target } => return Ok(Self::HardLink(path, strip_root(target)?)),
            Item::RegularInline { content, size, .. } if content.is_empty() && *size > 0 => {
                LeafContent::Regular(RegularFile::Sparse(*size))
            }
            Item::RegularInline { content, .. } => LeafContent::Regular(RegularFile::Inline(Box::from(&**content))),
            Item::Regular {
                size,
                fsverity_digest,
                path: object_path,
                ..
            } => {
                let object_id = match fsverity_digest {
                    Some(digest) => ObjectID::from_hex(digest)?,
                    None => ObjectID::from_object_pathname(object_path.as_os_str().as_bytes())?,
                };

                repo.repository()
                    .open_object(&object_id)
                    .map_err(|_| RepoError::NotFound)?;

                LeafContent::Regular(match fsverity_digest {
                    Some(_) => RegularFile::External(object_id, *size),
                    None => RegularFile::ExternalNoVerity(object_id, *size),
                })
            }
            Item::Device { rdev, .. } if SFlag::from_bits_truncate(entry.mode) & SFlag::S_IFMT == SFlag::S_IFBLK => {
                LeafContent::BlockDevice(*rdev)
            }
            Item::Device { rdev, .. } => LeafContent::CharacterDevice(*rdev),
            Item::Symlink { target, .. } => LeafContent::Symlink(Box::from(target.as_os_str())),
            Item::Fifo { .. } => LeafContent::Fifo,
            Item::Socket { .. } => LeafContent::Socket,
        };

        Ok(Self::Leaf(
            path,
            Leaf {
                stat: read_stat(&entry)?,
                content,
            },
        ))
    }

    pub fn write(&self, dumpfile: &mut String) -> Result<(), RepoError> {
        fn join_root(path: &Path) -> PathBuf {
            once(Component::RootDir)
                .chain(
                    path.components()
                        .filter(|component| !matches!(component, Component::CurDir | Component::RootDir)),
                )
                .collect()
        }

        match self {
            Self::Directory(path, stat) => write_directory(dumpfile, &join_root(path), stat, 1),
            Self::Leaf(path, leaf) => write_leaf(dumpfile, &join_root(path), &leaf.stat, &leaf.content, 1),
            Self::HardLink(path, original) => {
                write_hardlink(dumpfile, &join_root(path), join_root(original).as_os_str())
            }
        }
        .map_err(|_| RepoError::InvalidDumpfile)?;

        dumpfile.push('\n');

        Ok(())
    }
}
