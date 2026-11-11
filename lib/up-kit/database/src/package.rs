// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::collections::BTreeMap;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::str::from_utf8;

use composefs::generic_tree::{LeafContent, LeafId, Stat};
use composefs::tree::RegularFile;

use upac_composefs::Repo;
use upac_composefs::error::RepoError;
use upac_composefs::tree::builder::TreeBuilder;
use upac_composefs::tree::view::TreeView;

use upac_types::package::{PackageId, PackageMeta, PackageTrigger, TriggerPosition};

use super::error::DatabaseError;
use super::layout::meta::FILE_NAME as META_FILE_NAME;
use super::layout::package::{LOSSY_REPLACEMENT, SEPARATOR as PACKAGE_SEPARATOR};
use super::layout::store::{DIR_MODE, FILE_MODE, FORBIDDEN_CHARACTERS, LINK_MODE, LINK_ROOT, ROOT_DIR};
use super::layout::transaction::DIR as TRANSACTION_DIR;
use super::layout::trigger::{FILE_PREFIX as TRIGGER_FILE_PREFIX, XATTR_PREFIX as TRIGGER_XATTR_PREFIX};
use super::{PackageNodeId, PackageNodeKind};

pub struct PackageStore<TreeAccess> {
    tree: TreeAccess,
}

impl<'tree> PackageStore<TreeView<'tree>> {
    pub fn read(tree: TreeView<'tree>) -> Self {
        Self { tree }
    }

    pub fn contains(&self, package: &PackageId) -> bool {
        self.tree.check(Self::package_directory(package))
    }

    pub fn list(&self, repo: &Repo) -> Result<Vec<PackageMeta>, DatabaseError> {
        if !self.tree.check(ROOT_DIR) {
            return Ok(Vec::new());
        }

        self.tree
            .list(ROOT_DIR)?
            .into_iter()
            .filter(|directory_name| *directory_name != OsStr::new(TRANSACTION_DIR))
            .map(|directory_name| self.read_meta(repo, &Path::new(ROOT_DIR).join(directory_name)))
            .collect()
    }

    pub fn meta(&self, repo: &Repo, package: &PackageId) -> Result<Option<PackageMeta>, DatabaseError> {
        let directory = Self::package_directory(package);

        if !self.tree.check(&directory) {
            return Ok(None);
        }

        self.read_meta(repo, &directory).map(Some)
    }

    pub fn nodes(&self, package: &PackageId) -> Result<Vec<PackageNodeId>, DatabaseError> {
        self.tree
            .list(Self::package_directory(package))?
            .into_iter()
            .map(|node_name| node_name.to_str().ok_or(DatabaseError::InvalidNodeId))
            .filter(|node_name| {
                !matches!(node_name, Ok(name) if *name == META_FILE_NAME || name.starts_with(TRIGGER_FILE_PREFIX))
            })
            .map(|node_name| node_name?.parse())
            .collect()
    }

    pub fn node_leaf(&self, package: &PackageId, node: &PackageNodeId) -> Result<LeafId, DatabaseError> {
        if node.kind() == PackageNodeKind::Dir {
            return Err(DatabaseError::NodeKindMismatch);
        }

        let path = Self::package_directory(package).join(node.to_string());
        let (parent, name) = self
            .tree
            .filesystem()
            .root
            .split(path.as_os_str())
            .map_err(RepoError::from)?;

        Ok(parent.leaf_id(name).map_err(RepoError::from)?)
    }

    pub fn node_directory(&self, package: &PackageId, node: &PackageNodeId) -> Result<PathBuf, DatabaseError> {
        let path = Self::package_directory(package).join(node.to_string());

        if node.kind() != PackageNodeKind::Dir {
            return Err(DatabaseError::NodeKindMismatch);
        }

        let LeafContent::Symlink(target) = self.tree.get(&path)? else {
            return Err(DatabaseError::InvalidNodeId);
        };

        Path::new(&target)
            .strip_prefix(LINK_ROOT)
            .map(Path::to_path_buf)
            .map_err(|_| DatabaseError::InvalidNodeId)
    }

    fn read_meta(&self, repo: &Repo, directory: &Path) -> Result<PackageMeta, DatabaseError> {
        let LeafContent::Regular(file) = self.tree.get(directory.join(META_FILE_NAME))? else {
            return Err(DatabaseError::InvalidMeta);
        };

        let content = repo.read(&file)?;
        let content = from_utf8(&content).map_err(|_| DatabaseError::InvalidMeta)?;
        let mut meta: PackageMeta = toml::from_str(content).map_err(|_| DatabaseError::InvalidMeta)?;

        for file_name in self.tree.list(directory)? {
            let Some(trigger_name) = file_name
                .to_str()
                .and_then(|file_name| file_name.strip_prefix(TRIGGER_FILE_PREFIX))
            else {
                continue;
            };

            let stat = self.tree.get_metadata(directory.join(file_name))?;

            for position in TriggerPosition::ALL {
                let xattr_name = format!("{TRIGGER_XATTR_PREFIX}{position}");

                if stat.xattrs.contains_key(OsStr::new(&xattr_name)) {
                    meta.triggers.push(PackageTrigger {
                        position,
                        name: trigger_name.to_owned(),
                    });
                }
            }
        }

        Ok(meta)
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
        self.tree
            .insert_dir(Self::package_directory(package), Self::stat(DIR_MODE, BTreeMap::new()))?;

        Ok(())
    }

    pub fn insert(&mut self, package: &PackageId, node: &PackageNodeId, place: &Path) -> Result<(), DatabaseError> {
        let path = Self::package_directory(package).join(node.to_string());

        match node.kind() {
            PackageNodeKind::File | PackageNodeKind::Symlink => self.tree.insert_hard_link(path, place)?,
            PackageNodeKind::Dir => {
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

    pub fn set_meta(&mut self, repo: &Repo, package: &PackageId, meta: &PackageMeta) -> Result<(), DatabaseError> {
        let directory = Self::package_directory(package);

        let content = toml::to_string(meta).map_err(|_| DatabaseError::InvalidMeta)?;
        self.tree.insert(
            directory.join(META_FILE_NAME),
            LeafContent::Regular(repo.store(content.as_bytes())?),
            Self::stat(FILE_MODE, BTreeMap::new()),
        )?;

        let mut trigger_files: BTreeMap<&str, BTreeMap<Box<OsStr>, Box<[u8]>>> = BTreeMap::new();

        for trigger in &meta.triggers {
            if trigger.name.is_empty()
                || trigger
                    .name
                    .contains(|character: char| FORBIDDEN_CHARACTERS.contains(character))
            {
                return Err(DatabaseError::InvalidTriggers);
            }

            let xattr_name = format!("{TRIGGER_XATTR_PREFIX}{}", trigger.position);

            trigger_files
                .entry(&trigger.name)
                .or_default()
                .insert(Box::from(OsStr::new(&xattr_name)), Box::default());
        }

        for (trigger_name, xattrs) in trigger_files {
            self.tree.insert(
                directory.join(format!("{TRIGGER_FILE_PREFIX}{trigger_name}")),
                LeafContent::Regular(RegularFile::Inline(Box::default())),
                Self::stat(FILE_MODE, xattrs),
            )?;
        }

        Ok(())
    }

    pub fn remove(&mut self, package: &PackageId) -> Result<(), DatabaseError> {
        self.tree.remove(Self::package_directory(package))?;

        Ok(())
    }

    fn stat(mode: u32, xattrs: BTreeMap<Box<OsStr>, Box<[u8]>>) -> Stat {
        Stat {
            st_mode: mode,
            xattrs,
            ..Stat::uninitialized()
        }
    }
}

impl<TreeAccess> PackageStore<TreeAccess> {
    fn package_directory(package: &PackageId) -> PathBuf {
        fn sanitize_component(component: &str) -> String {
            component
                .replace(
                    |character: char| FORBIDDEN_CHARACTERS.contains(character),
                    LOSSY_REPLACEMENT,
                )
                .replace(PACKAGE_SEPARATOR, LOSSY_REPLACEMENT)
        }

        let mut directory_name = format!(
            "{}{}{}",
            sanitize_component(&package.name),
            PACKAGE_SEPARATOR,
            sanitize_component(&package.arch)
        );

        if let Some(arch_sub) = &package.arch_sub {
            directory_name.push_str(PACKAGE_SEPARATOR);
            directory_name.push_str(&sanitize_component(arch_sub));
        }

        PathBuf::from(ROOT_DIR).join(directory_name)
    }
}
