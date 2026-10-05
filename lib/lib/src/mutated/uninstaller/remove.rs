// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::path::Path;

use upac_types::CancelToken;
use upac_types::error::ErrorKind;
use upac_types::response::entry::FileEntryScope;

use upac_database::files::{FileStore, FileStoreMut};
use upac_database::meta::{MetaStore, MetaStoreMut};
use upac_database::triggers::TriggerStoreMut;

use upac_macro::stage;

use upac_orchestrator::context::Context;
use upac_orchestrator::stage::Stage;

use super::super::stages::WorkingPrefix;
use super::{Purge, RemovalTarget};

use crate::layout::prefix::DEFAULTS_DIR;

pub struct RemoveStage;

#[stage]
impl Stage<ErrorKind> for RemoveStage {
    fn run(
        &self, context: &mut Context, _cancel: &CancelToken, _progress: &dyn Fn(Option<&str>, u64, u64),
    ) -> Result<(), ErrorKind> {
        let target = context.take::<RemovalTarget>()?;
        let mut working = context.take::<WorkingPrefix>()?;
        let purge = context.get::<Purge>()?.0;

        let uuid = target.0;
        for entry in working.database.list_package_files(uuid)? {
            if entry.is_user && !purge {
                continue;
            }

            match entry.scope {
                FileEntryScope::Prefix => working.tree.remove(&entry.path)?,
                FileEntryScope::Config => working.tree.remove(Path::new(DEFAULTS_DIR).join(&entry.path))?,
            }

            if entry.is_user {
                working.database.remove_user_file(uuid, &entry.path)?;
            } else {
                working.database.remove_package_file(uuid, &entry.path)?;
            }
        }

        let meta = working.database.get_package_meta(uuid)?.ok_or(ErrorKind::NotFound)?;
        working
            .database
            .remove_package_meta(&meta.name, &meta.arch, meta.arch_sub.as_deref())?;
        working.database.remove_package_triggers(uuid)?;

        context.put(working);

        Ok(())
    }
}
