// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::fs::File;
use std::io::Read;

use composefs::erofs::reader::erofs_to_filesystem;
use composefs::tree::FileSystem;

use upac_types::package::ObjectID;

use self::builder::TreeBuilder;
use self::view::TreeView;

use crate::error::RepoError;
use crate::{Digest, Repo};

pub mod builder;
pub mod diff;
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
