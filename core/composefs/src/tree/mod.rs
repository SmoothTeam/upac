// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::fs::File;
use std::io::Read;

use composefs::dumpfile::dumpfile_to_filesystem;
use composefs::erofs::reader::erofs_to_filesystem;
use composefs::generic_tree::LeafContent;
use composefs::tree::{FileSystem, RegularFile};

use self::builder::TreeBuilder;
use self::view::TreeView;

use crate::error::RepoError;
use crate::{Digest, ObjectID, Repo};

pub mod builder;
pub mod diff;
pub mod entry;
pub mod overlay;
pub mod view;

#[derive(Clone)]
pub struct Tree {
    filesystem: FileSystem<ObjectID>,
}

impl Tree {
    pub fn open(repo: &Repo, digest: &Digest) -> Result<Self, RepoError> {
        let (image, _enable_verity) = repo.repository().open_image(&digest.to_hex())?;

        let mut data = Vec::new();
        File::from(image).read_to_end(&mut data)?;

        Ok(Self {
            filesystem: erofs_to_filesystem(&data)?,
        })
    }

    pub fn from_dumpfile(dumpfile: &str, repo: &Repo) -> Result<Self, RepoError> {
        let filesystem: FileSystem<ObjectID> =
            dumpfile_to_filesystem(dumpfile).map_err(|_| RepoError::InvalidDumpfile)?;

        for leaf in &filesystem.leaves {
            if let LeafContent::Regular(
                RegularFile::External(object_id, _) | RegularFile::ExternalNoVerity(object_id, _),
            ) = &leaf.content
            {
                repo.repository()
                    .open_object(object_id)
                    .map_err(|_| RepoError::NotFound)?;
            }
        }

        Ok(Self::from(filesystem))
    }

    pub fn edit(&self) -> TreeBuilder {
        TreeBuilder::from(self.filesystem.clone())
    }

    pub fn view(&self) -> TreeView<'_> {
        TreeView::new(&self.filesystem)
    }
}

impl From<FileSystem<ObjectID>> for Tree {
    fn from(mut filesystem: FileSystem<ObjectID>) -> Self {
        filesystem.compact();

        Self { filesystem }
    }
}
