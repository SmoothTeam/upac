// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_composefs::Repo;
use upac_composefs::tree::Tree;
use upac_composefs::tree::builder::TreeBuilder;

use upac_database::PackageId;

use upac_types::decoder::PackageTriggers;
use upac_types::package::PackageInfo;

use self::error::ImageError;
use self::index::ImageIndex;

mod index;
mod layout {
    include!(concat!(env!("OUT_DIR"), "/layout.rs"));
}

pub mod error;

pub struct ImageBuilder<'repo> {
    repo: &'repo Repo,
    builder: TreeBuilder,
    index: ImageIndex,
}

impl<'repo> ImageBuilder<'repo> {
    pub fn new(repo: &'repo Repo) -> Self {
        todo!()
    }

    pub fn edit(repo: &'repo Repo, image: &Tree) -> Result<Self, ImageError> {
        todo!()
    }

    pub fn install(
        &mut self, info: &PackageInfo, package: &Tree, triggers: &PackageTriggers,
    ) -> Result<PackageId, ImageError> {
        todo!()
    }

    pub fn uninstall(&mut self, package: &PackageId) -> Result<(), ImageError> {
        todo!()
    }

    pub fn commit(self) -> Result<Tree, ImageError> {
        todo!()
    }
}
