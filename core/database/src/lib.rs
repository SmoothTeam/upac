// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::fmt::{Display, Formatter, Result as FmtResult};
use std::os::unix::ffi::OsStrExt;
use std::path::Path;
use std::str::FromStr;

use composefs::fsverity::{FsVerityHashValue, compute_verity};
use composefs::generic_tree::{LeafContent, Stat};
use composefs::tree::{Inode, RegularFile};

use upac_composefs::ObjectID;
use upac_composefs::error::RepoError;
use upac_composefs::tree::view::TreeView;

use upac_types::package::PackageInfo;

use self::error::DatabaseError;
use self::layout::entry::{DIR_PREFIX, FILE_PREFIX, PERMISSION_MASK, SEPARATOR as ENTRY_SEPARATOR, SYMLINK_PREFIX};
use self::layout::package::{LOSSY_REPLACEMENT, SEPARATOR as PACKAGE_SEPARATOR};

pub mod error;
pub mod layout {
    include!(concat!(env!("OUT_DIR"), "/layout.rs"));
}
pub mod package;
pub mod transaction;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PackageEntryKind {
    File,
    Symlink,
    Dir,
}

impl Display for PackageEntryKind {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FmtResult {
        formatter.write_str(match self {
            PackageEntryKind::File => FILE_PREFIX,
            PackageEntryKind::Symlink => SYMLINK_PREFIX,
            PackageEntryKind::Dir => DIR_PREFIX,
        })
    }
}

impl FromStr for PackageEntryKind {
    type Err = DatabaseError;

    fn from_str(prefix: &str) -> Result<Self, Self::Err> {
        match prefix {
            FILE_PREFIX => Ok(PackageEntryKind::File),
            SYMLINK_PREFIX => Ok(PackageEntryKind::Symlink),
            DIR_PREFIX => Ok(PackageEntryKind::Dir),
            _ => Err(DatabaseError::InvalidEntryId),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PackageId(String);

impl PackageId {
    pub fn new(info: &PackageInfo) -> Self {
        fn sanitize_component(component: &str) -> String {
            component
                .replace(['/', '\0'], LOSSY_REPLACEMENT)
                .replace(PACKAGE_SEPARATOR, LOSSY_REPLACEMENT)
        }

        let mut directory_name = format!(
            "{}{}{}",
            sanitize_component(&info.name),
            PACKAGE_SEPARATOR,
            sanitize_component(&info.arch)
        );

        if let Some(arch_sub) = &info.arch_sub {
            directory_name.push_str(PACKAGE_SEPARATOR);
            directory_name.push_str(&sanitize_component(arch_sub));
        }

        Self(directory_name)
    }
}

impl Display for PackageId {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FmtResult {
        formatter.write_str(&self.0)
    }
}

impl FromStr for PackageId {
    type Err = DatabaseError;

    fn from_str(directory_name: &str) -> Result<Self, Self::Err> {
        let components: Vec<&str> = directory_name.split(PACKAGE_SEPARATOR).collect();

        if !(2..=3).contains(&components.len()) || components.iter().any(|component| component.is_empty()) {
            return Err(DatabaseError::InvalidPackageId);
        }

        Ok(Self(directory_name.to_owned()))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PackageEntryId {
    kind: PackageEntryKind,
    content: ObjectID,
    metadata: ObjectID,
}

impl PackageEntryId {
    pub fn new(kind: PackageEntryKind, content: ObjectID, stat: &Stat, place: &Path) -> Self {
        fn hash_metadata(stat: &Stat, place: &Path) -> ObjectID {
            let place = place.as_os_str().as_bytes();

            let mut serialized = Vec::new();
            serialized.extend_from_slice(&(place.len() as u64).to_le_bytes());
            serialized.extend_from_slice(place);
            serialized.extend_from_slice(&(stat.st_mode & PERMISSION_MASK).to_le_bytes());
            serialized.extend_from_slice(&stat.st_uid.to_le_bytes());
            serialized.extend_from_slice(&stat.st_gid.to_le_bytes());

            for (name, value) in &stat.xattrs {
                serialized.extend_from_slice(&(name.len() as u64).to_le_bytes());
                serialized.extend_from_slice(name.as_bytes());
                serialized.extend_from_slice(&(value.len() as u64).to_le_bytes());
                serialized.extend_from_slice(value);
            }

            compute_verity(&serialized)
        }

        Self {
            kind,
            content,
            metadata: hash_metadata(stat, place),
        }
    }

    pub fn from_leaf(
        content: &LeafContent<RegularFile<ObjectID>>, stat: &Stat, place: &Path,
    ) -> Result<Self, DatabaseError> {
        let (kind, content) = match content {
            LeafContent::Regular(RegularFile::External(object, _) | RegularFile::ExternalNoVerity(object, _)) => {
                (PackageEntryKind::File, object.clone())
            }
            LeafContent::Regular(RegularFile::Inline(bytes)) => (PackageEntryKind::File, compute_verity(bytes)),
            LeafContent::Symlink(target) => (PackageEntryKind::Symlink, compute_verity(target.as_bytes())),
            _ => return Err(DatabaseError::UnsupportedEntry),
        };

        Ok(Self::new(kind, content, stat, place))
    }

    pub fn from_directory(stat: &Stat, place: &Path) -> Self {
        Self::new(
            PackageEntryKind::Dir,
            compute_verity(place.as_os_str().as_bytes()),
            stat,
            place,
        )
    }

    pub fn from_place(view: TreeView<'_>, place: &Path) -> Result<Option<Self>, DatabaseError> {
        if !view.check(place) {
            return Ok(None);
        }

        let filesystem = view.filesystem();
        let (parent, name) = filesystem.root.split(place.as_os_str()).map_err(RepoError::from)?;

        match parent.lookup(name).ok_or(RepoError::NotFound)? {
            Inode::Directory(directory) => Ok(Some(Self::from_directory(&directory.stat, place))),
            Inode::Leaf(leaf_id, _) => {
                let leaf = filesystem.leaf(*leaf_id);

                Self::from_leaf(&leaf.content, &leaf.stat, place).map(Some)
            }
        }
    }

    pub fn kind(&self) -> PackageEntryKind {
        self.kind
    }

    pub fn content(&self) -> &ObjectID {
        &self.content
    }

    pub fn metadata(&self) -> &ObjectID {
        &self.metadata
    }
}

impl Display for PackageEntryId {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FmtResult {
        write!(
            formatter,
            "{}{}{}{}{}",
            self.kind,
            ENTRY_SEPARATOR,
            self.content.to_hex(),
            ENTRY_SEPARATOR,
            self.metadata.to_hex()
        )
    }
}

impl FromStr for PackageEntryId {
    type Err = DatabaseError;

    fn from_str(entry_name: &str) -> Result<Self, Self::Err> {
        let components: Vec<&str> = entry_name.split(ENTRY_SEPARATOR).collect();
        let [kind, content, metadata] = components.as_slice() else {
            return Err(DatabaseError::InvalidEntryId);
        };

        Ok(Self {
            kind: kind.parse()?,
            content: ObjectID::from_hex(content).map_err(|_| DatabaseError::InvalidEntryId)?,
            metadata: ObjectID::from_hex(metadata).map_err(|_| DatabaseError::InvalidEntryId)?,
        })
    }
}
