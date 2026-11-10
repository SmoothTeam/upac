// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use composefs::tree::Inode;

use upac_database::package::PackageStore;
use upac_database::{PackageEntryKind, PackageId};

use super::ImageBuilder;
use super::error::ImageError;
use super::layout::base::DIRS as BASE_DIRS;
use super::layout::image::CMP_DIR;

impl ImageBuilder {
    pub fn uninstall(&mut self, package: &PackageId) -> Result<(), ImageError> {
        let view = self.builder.view();
        let store = PackageStore::read(view);

        if !store.contains(package) {
            return Err(ImageError::PackageNotInstalled);
        }

        let mut shared = HashSet::new();

        for other in store.list()? {
            if other != *package {
                shared.extend(store.entries(&other)?);
            }
        }

        let mut directories: Vec<PathBuf> = Vec::new();
        let mut leaves = HashSet::new();

        for entry in store.entries(package)? {
            if shared.contains(&entry) {
                continue;
            }

            match entry.kind() {
                PackageEntryKind::Dir => directories.push(store.entry_directory(package, &entry)?),
                PackageEntryKind::File | PackageEntryKind::Symlink => {
                    leaves.insert(store.entry_leaf(package, &entry)?);
                }
            }
        }

        let places: Vec<PathBuf> = view
            .walk(CMP_DIR)?
            .filter_map(|(place, inode)| match inode {
                Inode::Leaf(leaf_id, _) if leaves.contains(leaf_id) => Some(place),
                _ => None,
            })
            .collect();

        for place in &places {
            self.builder.remove(place)?;
        }

        directories.retain(|directory| !BASE_DIRS.iter().any(|base_dir| Path::new(base_dir) == directory));
        directories.sort();

        for directory in directories.iter().rev() {
            if self
                .builder
                .view()
                .list(directory)
                .is_ok_and(|entries| entries.is_empty())
            {
                self.builder.remove(directory)?;
            }
        }

        PackageStore::write(&mut self.builder).remove(package)?;

        Ok(())
    }
}
