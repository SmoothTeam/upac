// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};

use composefs::fsverity::FsVerityHashValue;
use composefs::generic_tree::{LeafContent, Stat};
use composefs::tree::Inode;

use upac_database::layout::store::ROOT_DIR as PACKAGE_ROOT_DIR;
use upac_database::{PackageEntryId, PackageEntryKind};

use super::ImageBuilder;
use super::error::ImageError;
use super::layout::base::{DIRS as BASE_DIRS, SYMLINKS};
use super::layout::hidden::FILE_NAME as HIDDEN_FILE_NAME;
use super::layout::image::{
    BINARY_DIR, CMP_DIR, CONFIG_DEFAULTS_DIR, CONFIG_DEFAULTS_SOURCE_DIR, DIR_MODE, HASH_LENGTH, LIBRARY_DIR,
    LIBRARY_DIRS, LINK_MODE, NAME_SEPARATOR, PATH_DIRS, ROOT_DIR,
};

impl ImageBuilder {
    pub(crate) fn sync(&mut self) -> Result<(), ImageError> {
        self.check_base()?;
        self.clear_root()?;
        self.sync_defaults()?;
        self.sync_flat(BINARY_DIR, PATH_DIRS)?;
        self.sync_flat(LIBRARY_DIR, LIBRARY_DIRS)?;

        Ok(())
    }

    fn check_base(&self) -> Result<(), ImageError> {
        let view = self.builder.view();

        let is_dirs_present = BASE_DIRS
            .iter()
            .all(|base_dir| view.filesystem().root.get_directory(OsStr::new(base_dir)).is_ok());
        let is_symlinks_present = SYMLINKS.iter().all(|(name, target)| {
            matches!(view.get(name), Ok(LeafContent::Symlink(existing)) if existing.as_ref() == OsStr::new(target))
        });

        if is_dirs_present && is_symlinks_present {
            Ok(())
        } else {
            Err(ImageError::MissingBase)
        }
    }

    fn clear_root(&mut self) -> Result<(), ImageError> {
        let kept = [
            PACKAGE_ROOT_DIR,
            BINARY_DIR,
            LIBRARY_DIR,
            CONFIG_DEFAULTS_DIR,
            HIDDEN_FILE_NAME,
        ];
        let is_kept = |name: &OsStr| {
            kept.iter()
                .chain(BASE_DIRS)
                .chain(SYMLINKS.iter().map(|(link, _)| link))
                .any(|kept_name| OsStr::new(kept_name) == name)
        };

        let removed: Vec<OsString> = self
            .builder
            .view()
            .list("")?
            .into_iter()
            .filter(|name| !is_kept(name))
            .map(OsStr::to_owned)
            .collect();

        for name in removed {
            self.builder.remove(name)?;
        }

        Ok(())
    }

    fn sync_defaults(&mut self) -> Result<(), ImageError> {
        let source = Path::new(CMP_DIR).join(CONFIG_DEFAULTS_SOURCE_DIR);
        let view = self.builder.view();

        let root_stat = view.get_metadata(&source)?.clone();
        let nodes: Vec<(PathBuf, PathBuf, Option<Stat>)> = view
            .walk(&source)?
            .filter_map(|(place, inode)| {
                let target = Path::new(CONFIG_DEFAULTS_DIR).join(place.strip_prefix(&source).ok()?);
                let stat = match inode {
                    Inode::Directory(directory) => Some(directory.stat.clone()),
                    Inode::Leaf(..) => None,
                };

                Some((place, target, stat))
            })
            .collect();

        self.builder.remove(CONFIG_DEFAULTS_DIR)?;
        self.builder.insert_dir(CONFIG_DEFAULTS_DIR, root_stat)?;

        for (place, target, stat) in nodes {
            match stat {
                Some(stat) => self.builder.insert_dir(&target, stat)?,
                None => self.builder.insert_hard_link(&target, &place)?,
            }
        }

        Ok(())
    }

    fn sync_flat(&mut self, flat_dir: &str, source_dirs: &[(&str, &str)]) -> Result<(), ImageError> {
        let view = self.builder.view();
        let mut links: Vec<(PathBuf, PathBuf, PackageEntryKind)> = Vec::new();

        for (_, source_dir) in source_dirs {
            let directory = Path::new(CMP_DIR).join(source_dir);

            if !view.check(&directory) {
                continue;
            }

            for name in view.list(&directory)? {
                let place = directory.join(name);
                let Some(entry) = PackageEntryId::from_place(view, &place)? else {
                    continue;
                };

                if entry.kind() == PackageEntryKind::Dir {
                    continue;
                }

                let mut flat_name = name.to_owned();
                for hash in [entry.content().to_hex(), entry.metadata().to_hex()] {
                    flat_name.push(NAME_SEPARATOR);
                    flat_name.push(hash.get(..HASH_LENGTH as usize).unwrap_or(&hash));
                }

                links.push((Path::new(flat_dir).join(flat_name), place, entry.kind()));
            }
        }

        self.builder.remove(flat_dir)?;
        self.builder.insert_dir(
            flat_dir,
            Stat {
                st_mode: DIR_MODE,
                ..Stat::uninitialized()
            },
        )?;

        for (target, place, kind) in links {
            match kind {
                PackageEntryKind::File => self.builder.insert_hard_link(&target, &place)?,
                PackageEntryKind::Symlink => self.builder.insert(
                    &target,
                    LeafContent::Symlink(Path::new(ROOT_DIR).join(&place).into_os_string().into_boxed_os_str()),
                    Stat {
                        st_mode: LINK_MODE,
                        ..Stat::uninitialized()
                    },
                )?,
                PackageEntryKind::Dir => {}
            }
        }

        Ok(())
    }
}
