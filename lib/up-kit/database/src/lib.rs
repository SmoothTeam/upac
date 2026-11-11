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

use upac_composefs::error::RepoError;
use upac_composefs::tree::view::TreeView;

use upac_types::package::ObjectID;

use self::error::DatabaseError;
use self::layout::node::{DIR_PREFIX, FILE_PREFIX, PERMISSION_MASK, SEPARATOR as NODE_SEPARATOR, SYMLINK_PREFIX};

pub mod error;
pub mod layout {
    include!(concat!(env!("OUT_DIR"), "/layout.rs"));
}
pub mod package;
pub mod transaction;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PackageNodeKind {
    File,
    Symlink,
    Dir,
}

impl Display for PackageNodeKind {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FmtResult {
        formatter.write_str(match self {
            PackageNodeKind::File => FILE_PREFIX,
            PackageNodeKind::Symlink => SYMLINK_PREFIX,
            PackageNodeKind::Dir => DIR_PREFIX,
        })
    }
}

impl FromStr for PackageNodeKind {
    type Err = DatabaseError;

    fn from_str(prefix: &str) -> Result<Self, Self::Err> {
        match prefix {
            FILE_PREFIX => Ok(PackageNodeKind::File),
            SYMLINK_PREFIX => Ok(PackageNodeKind::Symlink),
            DIR_PREFIX => Ok(PackageNodeKind::Dir),
            _ => Err(DatabaseError::InvalidNodeId),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PackageNodeId {
    kind: PackageNodeKind,
    content: ObjectID,
    metadata: ObjectID,
}

impl PackageNodeId {
    pub fn new(kind: PackageNodeKind, content: ObjectID, stat: &Stat, place: &Path) -> Self {
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
                (PackageNodeKind::File, object.clone())
            }
            LeafContent::Regular(RegularFile::Inline(bytes)) => (PackageNodeKind::File, compute_verity(bytes)),
            LeafContent::Symlink(target) => (PackageNodeKind::Symlink, compute_verity(target.as_bytes())),
            _ => return Err(DatabaseError::UnsupportedNode),
        };

        Ok(Self::new(kind, content, stat, place))
    }

    pub fn from_directory(stat: &Stat, place: &Path) -> Self {
        Self::new(
            PackageNodeKind::Dir,
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

    pub fn kind(&self) -> PackageNodeKind {
        self.kind
    }

    pub fn content(&self) -> &ObjectID {
        &self.content
    }

    pub fn metadata(&self) -> &ObjectID {
        &self.metadata
    }
}

impl Display for PackageNodeId {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FmtResult {
        write!(
            formatter,
            "{}{}{}{}{}",
            self.kind,
            NODE_SEPARATOR,
            self.content.to_hex(),
            NODE_SEPARATOR,
            self.metadata.to_hex()
        )
    }
}

impl FromStr for PackageNodeId {
    type Err = DatabaseError;

    fn from_str(node_name: &str) -> Result<Self, Self::Err> {
        let components: Vec<&str> = node_name.split(NODE_SEPARATOR).collect();
        let [kind, content, metadata] = components.as_slice() else {
            return Err(DatabaseError::InvalidNodeId);
        };

        Ok(Self {
            kind: kind.parse()?,
            content: ObjectID::from_hex(content).map_err(|_| DatabaseError::InvalidNodeId)?,
            metadata: ObjectID::from_hex(metadata).map_err(|_| DatabaseError::InvalidNodeId)?,
        })
    }
}
