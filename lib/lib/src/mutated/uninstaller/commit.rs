// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::fs::{File, write};
use std::path::Path;

use composefs::fsverity::FsVerityHashValue;
use composefs::generic_tree::Stat;
use composefs::repository::ImportContext;

use upac_abi::hook::CancelToken;

use upac_types::TmpPath;
use upac_types::hook::ProgressEventBuilder;

use upac_composefs::error::RepoError;
use upac_composefs::file::FileHandle;
use upac_composefs::repository::commit_tree;

use upac_database::InMemory;
use upac_database::layout::database::DATABASE_PATH;

use upac_deploy::Deploy;

use upac_orchestrator::context::{Context, ctx_get, ctx_take};
use upac_orchestrator::stage::{NoRollback, RollbackGuard, Stage, StageResult};

use super::{NewState, UninstallError, WorkingState};

use crate::layout::database::UNINSTALL_SCRATCH_FILENAME;

pub struct CommitTransactionStage;

impl Stage<UninstallError> for CommitTransactionStage {
    fn run(
        &self, context: &mut Context, _cancel: &CancelToken, progress: ProgressEventBuilder,
    ) -> Result<(ProgressEventBuilder, StageResult, Box<dyn RollbackGuard>), UninstallError> {
        let working_state = ctx_take!(context, WorkingState);

        let tmp_path = ctx_get!(context, TmpPath);
        let deploy = ctx_get!(context, Deploy);

        let repository = deploy.open_repository()?;
        let mut tree = working_state.tree;

        let database_bytes = working_state.database.into_bytes()?;
        let database_scratch_path = Path::new(tmp_path.as_ref()).join(UNINSTALL_SCRATCH_FILENAME);
        write(&database_scratch_path, &database_bytes).map_err(RepoError::from)?;

        FileHandle::new(DATABASE_PATH).insert_file(
            &repository,
            &mut tree,
            &File::open(&database_scratch_path).map_err(RepoError::from)?,
            Stat::uninitialized(),
            &mut ImportContext::default(),
        )?;

        let digest = commit_tree(&repository, tree)?;

        context.put(NewState {
            prefix_digest: digest.to_hex(),
            removed_config_paths: working_state.removed_config_paths,
        });

        Ok((progress, StageResult::Advance, Box::new(NoRollback)))
    }
}
