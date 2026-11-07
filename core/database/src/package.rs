// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::collections::BTreeMap;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::str::from_utf8;

use composefs::generic_tree::{LeafContent, Stat};
use composefs::tree::RegularFile;

use upac_composefs::tree::Tree;
use upac_composefs::tree::builder::TreeBuilder;

use upac_types::decoder::{PackageTrigger, PackageTriggers};

use super::error::DatabaseError;
use super::layout::store::{DIR_MODE, FILE_MODE, LINK_MODE, LINK_ROOT, ROOT_DIR};
use super::layout::transaction::DIR as TRANSACTION_DIR;
use super::layout::trigger::{FILE_NAME, FORMAT_XATTR, LINE_SEPARATOR, SEPARATOR, XATTR};
use super::{PackageEntryId, PackageEntryKind, PackageId};

pub struct PackageStore<TreeAccess> {
    tree: TreeAccess,
}

impl<'tree> PackageStore<&'tree Tree> {
    pub fn read(tree: &'tree Tree) -> Self {
        Self { tree }
    }

    pub fn list(&self) -> Result<Vec<PackageId>, DatabaseError> {
        if !self.tree.check(ROOT_DIR) {
            return Ok(Vec::new());
        }

        self.tree
            .list(ROOT_DIR)?
            .into_iter()
            .filter(|entry_name| *entry_name != OsStr::new(TRANSACTION_DIR))
            .map(|entry_name| entry_name.to_str().ok_or(DatabaseError::InvalidPackageId)?.parse())
            .collect()
    }

    pub fn entries(&self, package: &PackageId) -> Result<Vec<PackageEntryId>, DatabaseError> {
        self.tree
            .list(PathBuf::from(ROOT_DIR).join(package.to_string()))?
            .into_iter()
            .filter(|entry_name| *entry_name != OsStr::new(FILE_NAME))
            .map(|entry_name| entry_name.to_str().ok_or(DatabaseError::InvalidEntryId)?.parse())
            .collect()
    }

    pub fn triggers(&self, package: &PackageId) -> Result<Option<PackageTriggers>, DatabaseError> {
        let path = PathBuf::from(ROOT_DIR).join(package.to_string()).join(FILE_NAME);

        if !self.tree.check(&path) {
            return Ok(None);
        }

        let stat = self.tree.get_metadata(&path)?;
        let format = self.xattr_text(stat, FORMAT_XATTR)?;
        let triggers = self
            .xattr_text(stat, XATTR)?
            .split(LINE_SEPARATOR)
            .filter(|line| !line.is_empty())
            .map(|line| self.trigger_parse(line))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Some(PackageTriggers {
            format: format.to_owned(),
            triggers,
        }))
    }
}

impl<'tree> PackageStore<&'tree mut TreeBuilder> {
    pub fn write(builder: &'tree mut TreeBuilder) -> Self {
        Self { tree: builder }
    }

    pub fn init(&mut self) -> Result<(), DatabaseError> {
        self.tree.insert_dir(ROOT_DIR, Self::stat(DIR_MODE, BTreeMap::new()))?;

        Ok(())
    }

    pub fn create(&mut self, package: &PackageId) -> Result<(), DatabaseError> {
        self.tree.insert_dir(
            PathBuf::from(ROOT_DIR).join(package.to_string()),
            Self::stat(DIR_MODE, BTreeMap::new()),
        )?;

        Ok(())
    }

    pub fn insert(&mut self, package: &PackageId, entry: &PackageEntryId, place: &Path) -> Result<(), DatabaseError> {
        let path = PathBuf::from(ROOT_DIR)
            .join(package.to_string())
            .join(entry.to_string());

        match entry.kind() {
            PackageEntryKind::File | PackageEntryKind::Symlink => self.tree.insert_hard_link(path, place)?,
            PackageEntryKind::Dir => {
                let target = Path::new(LINK_ROOT).join(place);

                self.tree.insert(
                    path,
                    LeafContent::Symlink(target.into_os_string().into_boxed_os_str()),
                    Self::stat(LINK_MODE, BTreeMap::new()),
                )?
            }
        }

        Ok(())
    }

    pub fn set_triggers(&mut self, package: &PackageId, triggers: &PackageTriggers) -> Result<(), DatabaseError> {
        let mut lines = String::new();
        for trigger in &triggers.triggers {
            if trigger.name.contains(LINE_SEPARATOR) {
                return Err(DatabaseError::InvalidTriggers);
            }

            lines.push_str(&trigger.position.to_string());
            lines.push_str(SEPARATOR);
            lines.push_str(&trigger.name);
            lines.push_str(LINE_SEPARATOR);
        }

        let xattrs = BTreeMap::from([
            (OsStr::new(XATTR).into(), lines.into_bytes().into_boxed_slice()),
            (OsStr::new(FORMAT_XATTR).into(), triggers.format.as_bytes().into()),
        ]);

        self.tree.insert(
            PathBuf::from(ROOT_DIR).join(package.to_string()).join(FILE_NAME),
            LeafContent::Regular(RegularFile::Inline(Box::default())),
            Self::stat(FILE_MODE, xattrs),
        )?;

        Ok(())
    }

    pub fn remove(&mut self, package: &PackageId) -> Result<(), DatabaseError> {
        self.tree.remove(PathBuf::from(ROOT_DIR).join(package.to_string()))?;

        Ok(())
    }

    fn stat(mode: u32, xattrs: BTreeMap<Box<OsStr>, Box<[u8]>>) -> Stat {
        Stat {
            st_mode: mode,
            st_uid: 0,
            st_gid: 0,
            st_mtim_sec: 0,
            st_mtim_nsec: 0,
            xattrs,
        }
    }
}

impl<TreeAccess> PackageStore<TreeAccess> {
    fn xattr_text<'stat>(&self, stat: &'stat Stat, name: &str) -> Result<&'stat str, DatabaseError> {
        let value = stat
            .xattrs
            .get(OsStr::new(name))
            .ok_or(DatabaseError::InvalidTriggers)?;

        from_utf8(value).map_err(|_| DatabaseError::InvalidTriggers)
    }

    fn trigger_parse(&self, line: &str) -> Result<PackageTrigger, DatabaseError> {
        let (position, name) = line.split_once(SEPARATOR).ok_or(DatabaseError::InvalidTriggers)?;

        Ok(PackageTrigger {
            position: position.parse().map_err(|_| DatabaseError::InvalidTriggers)?,
            name: name.to_owned(),
        })
    }
}
