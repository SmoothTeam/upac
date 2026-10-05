// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::cmp::Ordering;
use std::fs::remove_dir_all;
use std::path::{Path, PathBuf};

use composefs::generic_tree::Stat;

use upac_types::CancelToken;
use upac_types::error::ErrorKind;
use upac_types::response::entry::{FileEntry, FileEntryScope};

use upac_composefs::tree::Tree;

use upac_database::files::{FileStore, FileStoreMut};
use upac_database::meta::{MetaStore, MetaStoreMut};
use upac_database::triggers::TriggerStoreMut;

use upac_macro::stage;

use upac_orchestrator::context::Context;
use upac_orchestrator::stage::Stage;

use super::super::stages::{UnpackedPackage, WorkingPrefix};
use super::AllowDowngrade;

use crate::layout::prefix::{DEFAULTS_DIR, PACKAGE_CONFIG_DIR, PACKAGE_PREFIX_DIR};

pub struct ImportStage;

#[stage]
impl Stage<ErrorKind> for ImportStage {
    fn run(
        &self, context: &mut Context, cancel: &CancelToken, _progress: &dyn Fn(Option<&str>, u64, u64),
    ) -> Result<(), ErrorKind> {
        let package = context.take::<UnpackedPackage>()?;
        let mut working = context.take::<WorkingPrefix>()?;
        let allow_downgrade = context.get::<AllowDowngrade>()?.0;

        let meta = &package.temp.meta;
        let uuid = working
            .database
            .find_package_uuid(&meta.name, &meta.arch, meta.arch_sub.as_deref())?
            .ok_or(ErrorKind::NotFound)?;
        let installed_meta = working.database.get_package_meta(uuid)?.ok_or(ErrorKind::NotFound)?;

        match meta.version.cmp(&installed_meta.version) {
            Ordering::Equal => return Err(ErrorKind::AlreadyExists),
            Ordering::Less if !allow_downgrade => return Err(ErrorKind::InvalidEntry),
            Ordering::Less | Ordering::Greater => {}
        }

        for entry in working.database.list_package_files(uuid)? {
            match entry.scope {
                FileEntryScope::Prefix => working.tree.remove(&entry.path)?,
                FileEntryScope::Config => working.tree.remove(Path::new(DEFAULTS_DIR).join(&entry.path))?,
            }
            working.database.remove_package_file(uuid, &entry.path)?;
        }

        let source_root = Path::new(&package.temp.temp_package_path);
        let prefix_files =
            Self::import_if_present(&mut working.tree, "", &source_root.join(PACKAGE_PREFIX_DIR), cancel)?;
        let config_files = Self::import_if_present(
            &mut working.tree,
            DEFAULTS_DIR,
            &source_root.join(PACKAGE_CONFIG_DIR),
            cancel,
        )?;

        working.database.update_package_meta(meta)?;
        working.database.set_package_triggers(uuid, &package.triggers)?;

        let entries = prefix_files
            .into_iter()
            .map(|path| (path, FileEntryScope::Prefix))
            .chain(config_files.into_iter().map(|path| (path, FileEntryScope::Config)));
        for (path, scope) in entries {
            working.database.insert_package_file(
                uuid,
                &FileEntry {
                    path: path.to_string_lossy().into_owned(),
                    is_user: false,
                    scope,
                },
            )?;
        }

        remove_dir_all(source_root)?;

        context.put(working);

        Ok(())
    }
}

impl ImportStage {
    fn import_if_present(
        tree: &mut Tree, target: &str, source: &Path, cancel: &CancelToken,
    ) -> Result<Vec<PathBuf>, ErrorKind> {
        if !source.is_dir() {
            return Ok(Vec::new());
        }

        if !target.is_empty() && !tree.contains(target) {
            tree.insert_dir(target, Stat::uninitialized())?;
        }

        Ok(tree.import_dir(target, source, cancel, &mut |_| {})?)
    }
}
