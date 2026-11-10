// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::ffi::OsStr;
use std::path::{Component, Path, PathBuf};

use composefs::generic_tree::{LeafContent, Stat};
use composefs::tree::Leaf;

use upac_composefs::ObjectID;
use upac_composefs::error::RepoError;
use upac_composefs::tree::entry::TreeEntry;

use upac_database::error::DatabaseError;
use upac_database::package::PackageStore;
use upac_database::{PackageEntryId, PackageEntryKind, PackageId};

use upac_types::PackageInfo;
use upac_types::decoder::PackageTriggers;

use super::ImageBuilder;
use super::error::ImageError;
use super::layout::base::DIRS as BASE_DIRS;
use super::layout::image::{CMP_DIR, CONFIG_DEFAULTS_SOURCE_DIR, PACKAGE_CONFIG_DIR, ROOT_DIR};

impl ImageBuilder {
    pub fn install(
        &mut self, info: &PackageInfo, entries: &[TreeEntry], triggers: &PackageTriggers,
    ) -> Result<PackageId, ImageError> {
        fn relocate(package_path: &Path) -> PathBuf {
            let Ok(rest) = package_path.strip_prefix(PACKAGE_CONFIG_DIR) else {
                return Path::new(CMP_DIR).join(package_path);
            };
            let defaults = Path::new(CMP_DIR).join(CONFIG_DEFAULTS_SOURCE_DIR);

            if rest.as_os_str().is_empty() {
                defaults
            } else {
                defaults.join(rest)
            }
        }

        let package_id = PackageId::new(info);

        if PackageStore::read(self.builder.view()).contains(&package_id) {
            return Err(ImageError::PackageExists);
        }

        let (hard_links, files): (Vec<_>, Vec<_>) = entries
            .iter()
            .partition(|tree_entry| matches!(tree_entry, TreeEntry::HardLink(..)));
        let (configs, rest): (Vec<_>, Vec<_>) = files
            .into_iter()
            .partition(|tree_entry| tree_entry.path().starts_with(PACKAGE_CONFIG_DIR));

        let mut placed = Vec::new();

        for tree_entry in rest.into_iter().chain(configs).chain(hard_links) {
            let place = relocate(tree_entry.path());

            let entry = match tree_entry {
                TreeEntry::Directory(_, stat) => self.place_directory(&place, stat)?,
                TreeEntry::Leaf(package_path, leaf) => match self.place_leaf(package_path, &place, leaf)? {
                    Some(entry) => entry,
                    None => continue,
                },
                TreeEntry::HardLink(_, original) => self.place_hard_link(&place, &relocate(original))?,
            };

            placed.push((entry, place));
        }

        let mut store = PackageStore::write(&mut self.builder);
        store.create(&package_id)?;

        for (entry, place) in &placed {
            store.insert(&package_id, entry, place)?;
        }

        store.set_triggers(&package_id, triggers)?;

        Ok(package_id)
    }

    fn place_directory(&mut self, place: &Path, stat: &Stat) -> Result<PackageEntryId, ImageError> {
        let entry = PackageEntryId::from_directory(stat, place);

        if BASE_DIRS.iter().any(|base_dir| Path::new(base_dir) == place) {
            return Ok(entry);
        }

        let inserted = self.builder.try_insert_dir(place, stat.clone());
        self.resolve_insertion(place, &entry, inserted)?;

        Ok(entry)
    }

    fn place_leaf(
        &mut self, package_path: &Path, place: &Path, leaf: &Leaf<ObjectID>,
    ) -> Result<Option<PackageEntryId>, ImageError> {
        fn rewrite_symlink(package_path: &Path, target: &Path) -> PathBuf {
            let config_root = [PACKAGE_CONFIG_DIR, CONFIG_DEFAULTS_SOURCE_DIR]
                .into_iter()
                .map(Path::new)
                .find(|root| package_path.starts_with(root));

            let (Some(config_root), Some(parent), false) = (config_root, package_path.parent(), target.is_absolute())
            else {
                return target.to_path_buf();
            };

            let mut resolved: Vec<&OsStr> = parent.iter().collect();

            for component in target.components() {
                match component {
                    Component::ParentDir => {
                        resolved.pop();
                    }
                    Component::Normal(name) => resolved.push(name),
                    Component::CurDir | Component::RootDir | Component::Prefix(_) => {}
                }
            }

            let resolved: PathBuf = resolved.into_iter().collect();

            if resolved.starts_with(config_root) {
                return target.to_path_buf();
            }

            Path::new(ROOT_DIR).join(resolved)
        }

        let content = match &leaf.content {
            LeafContent::Symlink(target) => LeafContent::Symlink(
                rewrite_symlink(package_path, Path::new(target))
                    .into_os_string()
                    .into_boxed_os_str(),
            ),
            content => content.clone(),
        };

        let entry = match PackageEntryId::from_leaf(&content, &leaf.stat, place) {
            Ok(entry) => entry,
            Err(DatabaseError::UnsupportedEntry) => return Ok(None),
            Err(error) => return Err(error.into()),
        };

        let inserted = self.builder.try_insert(place, content, leaf.stat.clone());
        self.resolve_insertion(place, &entry, inserted)?;

        Ok(Some(entry))
    }

    fn place_hard_link(&mut self, place: &Path, original: &Path) -> Result<PackageEntryId, ImageError> {
        let view = self.builder.view();
        let entry = PackageEntryId::from_leaf(&view.get(original)?, view.get_metadata(original)?, place)?;

        let inserted = if view.check(place) {
            Err(RepoError::AlreadyExists)
        } else {
            self.builder.insert_hard_link(place, original)
        };
        self.resolve_insertion(place, &entry, inserted)?;

        Ok(entry)
    }

    fn resolve_insertion(
        &self, place: &Path, entry: &PackageEntryId, inserted: Result<(), RepoError>,
    ) -> Result<(), ImageError> {
        match inserted {
            Ok(()) => Ok(()),
            Err(RepoError::AlreadyExists) => match PackageEntryId::from_place(self.builder.view(), place)? {
                Some(existing) if existing == *entry => Ok(()),
                Some(existing) if existing.kind() == PackageEntryKind::Dir && entry.kind() == PackageEntryKind::Dir => {
                    Err(ImageError::DirectoryConflict)
                }
                _ => Err(ImageError::PathConflict),
            },
            Err(RepoError::NotFound) => Err(ImageError::MissingParent),
            Err(error) => Err(error.into()),
        }
    }
}
