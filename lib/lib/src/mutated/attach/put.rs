// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use composefs::generic_tree::Stat;

use upac_types::CancelToken;
use upac_types::error::ErrorKind;
use upac_types::response::entry::FileEntry;

use upac_database::files::FileStoreMut;

use upac_macro::stage;

use upac_orchestrator::context::Context;
use upac_orchestrator::stage::Stage;

use super::super::stages::{AttachItem, FileOwner, WorkingPrefix};

pub struct PutStage;

#[stage]
impl Stage<ErrorKind> for PutStage {
    fn run(
        &self, context: &mut Context, _cancel: &CancelToken, _progress: &dyn Fn(Option<&str>, u64, u64),
    ) -> Result<(), ErrorKind> {
        let item = context.take::<AttachItem>()?;
        let mut working = context.take::<WorkingPrefix>()?;
        let owner = context.get::<FileOwner>()?.0;

        let mut missing_ancestors: Vec<_> = item
            .target
            .tree_path
            .ancestors()
            .skip(1)
            .filter(|ancestor| !ancestor.as_os_str().is_empty() && !working.tree.contains(ancestor))
            .collect();
        missing_ancestors.reverse();
        for ancestor in missing_ancestors {
            working.tree.insert_dir(ancestor, Stat::uninitialized())?;
        }

        working.tree.import_path(&item.target.tree_path, &item.source)?;
        working.database.insert_package_file(
            owner,
            &FileEntry {
                path: item.target.entry_path,
                is_user: true,
                scope: item.target.scope,
            },
        )?;

        context.put(working);

        Ok(())
    }
}
