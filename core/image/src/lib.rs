// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::collections::BTreeMap;
use std::ffi::OsStr;

use composefs::generic_tree::{LeafContent, Stat};
use composefs::tree::RegularFile;

use upac_composefs::error::RepoError;
use upac_composefs::tree::Tree;
use upac_composefs::tree::builder::TreeBuilder;

use upac_database::package::PackageStore;
use upac_database::transaction::TransactionStore;

use self::error::ImageError;
use self::layout::base::{DIR_MODE as BASE_DIR_MODE, DIRS as BASE_DIRS, LINK_MODE as BASE_LINK_MODE, SYMLINKS};
use self::layout::hidden::{FILE_NAME as HIDDEN_FILE_NAME, LINE_SEPARATOR, SYMLINK_XATTR};
use self::layout::image::{CMP_DIR, FILE_MODE};

mod install;
mod layout {
    include!(concat!(env!("OUT_DIR"), "/layout.rs"));
}
mod sync;
mod uninstall;

pub mod error;

pub struct ImageBuilder {
    builder: TreeBuilder,
}

impl ImageBuilder {
    pub fn new() -> Result<Self, ImageError> {
        let mut builder = TreeBuilder::new();

        PackageStore::write(&mut builder).init()?;
        TransactionStore::write(&mut builder).init()?;

        for base_dir in BASE_DIRS {
            builder.insert_dir(
                base_dir,
                Stat {
                    st_mode: BASE_DIR_MODE,
                    ..Stat::uninitialized()
                },
            )?;
        }

        let mut hidden_names = String::new();

        for (name, target) in SYMLINKS {
            builder.insert(
                name,
                LeafContent::Symlink(OsStr::new(target).into()),
                Stat {
                    st_mode: BASE_LINK_MODE,
                    ..Stat::uninitialized()
                },
            )?;

            hidden_names.push_str(name);
            hidden_names.push_str(LINE_SEPARATOR);
        }

        builder.update_metadata(
            CMP_DIR,
            Stat {
                st_mode: BASE_DIR_MODE,
                xattrs: BTreeMap::from([(OsStr::new(SYMLINK_XATTR).into(), hidden_names.as_bytes().into())]),
                ..Stat::uninitialized()
            },
        )?;

        builder.insert(
            HIDDEN_FILE_NAME,
            LeafContent::Regular(RegularFile::Inline(hidden_names.into_bytes().into_boxed_slice())),
            Stat {
                st_mode: FILE_MODE,
                ..Stat::uninitialized()
            },
        )?;

        Ok(Self::from(builder))
    }

    pub fn repair(&mut self) -> Result<(), ImageError> {
        for base_dir in BASE_DIRS {
            let inserted = self.builder.try_insert_dir(
                base_dir,
                Stat {
                    st_mode: BASE_DIR_MODE,
                    ..Stat::uninitialized()
                },
            );

            match inserted {
                Ok(()) => {}
                Err(RepoError::AlreadyExists)
                    if self
                        .builder
                        .view()
                        .filesystem()
                        .root
                        .get_directory(OsStr::new(base_dir))
                        .is_ok() => {}
                Err(RepoError::AlreadyExists) => return Err(ImageError::MissingBase),
                Err(error) => return Err(error.into()),
            }
        }

        let mut hidden_names = String::new();

        for (name, target) in SYMLINKS {
            match self.builder.view().get(name) {
                Ok(LeafContent::Symlink(_)) | Err(RepoError::NotFound) => self.builder.insert(
                    name,
                    LeafContent::Symlink(OsStr::new(target).into()),
                    Stat {
                        st_mode: BASE_LINK_MODE,
                        ..Stat::uninitialized()
                    },
                )?,
                _ => return Err(ImageError::MissingBase),
            }

            hidden_names.push_str(name);
            hidden_names.push_str(LINE_SEPARATOR);
        }

        self.builder.update_metadata(
            CMP_DIR,
            Stat {
                st_mode: BASE_DIR_MODE,
                xattrs: BTreeMap::from([(OsStr::new(SYMLINK_XATTR).into(), hidden_names.as_bytes().into())]),
                ..Stat::uninitialized()
            },
        )?;

        self.builder.insert(
            HIDDEN_FILE_NAME,
            LeafContent::Regular(RegularFile::Inline(hidden_names.into_bytes().into_boxed_slice())),
            Stat {
                st_mode: FILE_MODE,
                ..Stat::uninitialized()
            },
        )?;

        Ok(())
    }

    pub fn commit(mut self) -> Result<Tree, ImageError> {
        self.sync()?;

        Ok(self.builder.commit())
    }
}

impl From<Tree> for ImageBuilder {
    fn from(image: Tree) -> Self {
        Self::from(TreeBuilder::from(image))
    }
}

impl From<TreeBuilder> for ImageBuilder {
    fn from(builder: TreeBuilder) -> Self {
        Self { builder }
    }
}
