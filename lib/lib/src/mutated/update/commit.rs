// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::fs::{File, remove_file, write};
use std::path::Path;

use composefs::generic_tree::Stat;

use upac_types::CancelToken;
use upac_types::error::ErrorKind;
use upac_types::transaction::{Transaction, TransactionKind};

use upac_database::layout::database::DATABASE_PATH;
use upac_database::transaction::TransactionStoreMut;

use upac_deploy::Sysroot;

use upac_macro::stage;

use upac_orchestrator::context::Context;
use upac_orchestrator::stage::Stage;

use super::super::TmpPath;
use super::super::stages::{CommitInfo, NewDefaults, NewPrefix, WorkingPrefix};

use crate::layout::database::UPDATE_SCRATCH_FILENAME;
use crate::layout::prefix::DEFAULTS_DIR;

pub struct CommitStage;

#[stage]
impl Stage<ErrorKind> for CommitStage {
    fn run(
        &self, context: &mut Context, _cancel: &CancelToken, _progress: &dyn Fn(Option<&str>, u64, u64),
    ) -> Result<(), ErrorKind> {
        let working = context.take::<WorkingPrefix>()?;
        let commit_info = context.get::<CommitInfo>()?;

        let transaction = Transaction::new(
            Some(working.parent_transaction),
            TransactionKind::Update,
            commit_info.subject.clone(),
            commit_info.message.clone(),
        );

        let mut database = working.database;
        database.set_transaction(&transaction)?;

        let database_path = Path::new(&context.get::<TmpPath>()?.0).join(UPDATE_SCRATCH_FILENAME);
        write(&database_path, database.into_bytes()?)?;

        let mut tree = working.tree;
        let inserted = File::open(&database_path)
            .map_err(ErrorKind::from)
            .and_then(|database_file| Ok(tree.insert_file(DATABASE_PATH, &database_file, Stat::uninitialized())?));
        remove_file(&database_path)?;
        inserted?;

        let new_defaults = if tree.contains(DEFAULTS_DIR) {
            tree.copy_tree(DEFAULTS_DIR)?
        } else {
            context.get::<Sysroot>()?.repo().empty_tree()
        };
        let digest = tree.commit()?;

        context.put(NewPrefix { digest, transaction });
        context.put(NewDefaults(new_defaults));

        Ok(())
    }
}
