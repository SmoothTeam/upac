// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::collections::BTreeMap;
use std::ffi::OsStr;
use std::path::PathBuf;
use std::str::from_utf8;

use composefs::generic_tree::{LeafContent, Stat};
use composefs::tree::RegularFile;

use upac_composefs::tree::Tree;
use upac_composefs::tree::builder::TreeBuilder;

use upac_types::decoder::{PackageTrigger, PackageTriggers};

use super::PackageId;
use super::error::DatabaseError;
use super::layout::store::{FILE_MODE, ROOT_DIR};
use super::layout::trigger::{FILE_NAME, FORMAT_XATTR, LINE_SEPARATOR, SEPARATOR, XATTR};

pub struct PackageTriggerStore<TreeAccess> {
    tree: TreeAccess,
}

impl<'tree> PackageTriggerStore<&'tree Tree> {
    pub fn read(tree: &'tree Tree) -> Self {
        Self { tree }
    }

    pub fn get(&self, package: &PackageId) -> Result<Option<PackageTriggers>, DatabaseError> {
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
            .map(|str| self.trigger_parse(str))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Some(PackageTriggers {
            format: format.to_owned(),
            triggers,
        }))
    }
}

impl<'tree> PackageTriggerStore<&'tree mut TreeBuilder> {
    pub fn write(builder: &'tree mut TreeBuilder) -> Self {
        Self { tree: builder }
    }

    pub fn set(&mut self, package: &PackageId, triggers: &PackageTriggers) -> Result<(), DatabaseError> {
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

        let stat = Stat {
            st_mode: FILE_MODE,
            st_uid: 0,
            st_gid: 0,
            st_mtim_sec: 0,
            st_mtim_nsec: 0,
            xattrs: BTreeMap::from([
                (OsStr::new(XATTR).into(), lines.into_bytes().into_boxed_slice()),
                (OsStr::new(FORMAT_XATTR).into(), triggers.format.as_bytes().into()),
            ]),
        };

        let path = PathBuf::from(ROOT_DIR).join(package.to_string()).join(FILE_NAME);
        let leaf = LeafContent::Regular(RegularFile::Inline(Box::default()));

        self.tree.insert(path, leaf, stat)?;

        Ok(())
    }

    pub fn remove(&mut self, package: &PackageId) -> Result<(), DatabaseError> {
        let path = PathBuf::from(ROOT_DIR).join(package.to_string()).join(FILE_NAME);

        self.tree.remove(path)?;

        Ok(())
    }
}

impl<TreeAccess> PackageTriggerStore<TreeAccess> {
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
