// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::ffi::OsStr;
use std::fs::{File, Metadata, read_dir, read_link};
use std::os::unix::fs::{FileTypeExt, MetadataExt};
use std::path::Path;

use composefs::generic_tree::LeafContent;
use composefs::tree::RegularFile;

use upac_types::CancelToken;

use super::Tree;
use super::builder::TreeBuilder;

use crate::error::RepoError;
use crate::layout::deployment::OVERLAY_OPAQUE_XATTR;
use crate::stat::ToStat;
use crate::{ObjectID, Repo};

impl Tree {
    pub fn from_host(
        repo: &Repo, dir: &Path, cancel: &CancelToken, on_entry: &mut dyn FnMut(&Path),
    ) -> Result<Tree, RepoError> {
        let mut builder = TreeBuilder::new(repo);
        builder.insert_host_entries(Path::new(""), dir, cancel, on_entry)?;

        Ok(builder.build())
    }
}

impl TreeBuilder {
    fn insert_host_entries(
        &mut self, path: &Path, dir: &Path, cancel: &CancelToken, on_entry: &mut dyn FnMut(&Path),
    ) -> Result<(), RepoError> {
        for entry in read_dir(dir)? {
            if cancel.is_cancelled() {
                return Err(RepoError::Cancelled);
            }

            let entry = entry?;
            let child = path.join(entry.file_name());
            let source = entry.path();
            let metadata = entry.metadata()?;
            on_entry(&child);

            if metadata.is_dir() {
                let mut stat = metadata.to_stat();
                if let Some(value) = xattr::get(&source, OVERLAY_OPAQUE_XATTR)? {
                    stat.xattrs
                        .insert(OsStr::new(OVERLAY_OPAQUE_XATTR).into(), value.into_boxed_slice());
                }

                self.insert_dir(&child, stat)?;
                self.insert_host_entries(&child, &source, cancel, on_entry)?;
            } else {
                let content = self.host_leaf(&source, &metadata)?;
                self.insert(&child, content, metadata.to_stat())?;
            }
        }

        Ok(())
    }

    fn host_leaf(&self, source: &Path, metadata: &Metadata) -> Result<LeafContent<RegularFile<ObjectID>>, RepoError> {
        Ok(match metadata.file_type() {
            file_type if file_type.is_symlink() => {
                LeafContent::Symlink(read_link(source)?.into_os_string().into_boxed_os_str())
            }
            file_type if file_type.is_char_device() => LeafContent::CharacterDevice(metadata.rdev()),
            file_type if file_type.is_block_device() => LeafContent::BlockDevice(metadata.rdev()),
            file_type if file_type.is_fifo() => LeafContent::Fifo,
            file_type if file_type.is_socket() => LeafContent::Socket,
            _ => LeafContent::Regular(self.repo.store_file(&File::open(source)?)?),
        })
    }
}
