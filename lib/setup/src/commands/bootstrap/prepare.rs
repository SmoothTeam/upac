// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::fs::metadata;
use std::path::Path;

use tempfile::TempDir;

use upac::orchestrator::context::{Context, ctx_get};
use upac::orchestrator::stage::{NoRollback, RollbackGuard, Stage, StageResult};

use upac_abi::hook::CancelToken;

use upac_types::hook::ProgressEventBuilder;

use super::error::BootstrapError;
use super::{RequestedSource, ResolvedSourceDir};

use crate::archive::SourceArchive;

pub struct PrepareSourceStage;

impl Stage<BootstrapError> for PrepareSourceStage {
    fn run(
        &self, context: &mut Context, _cancel: &CancelToken, progress: ProgressEventBuilder,
    ) -> Result<(ProgressEventBuilder, StageResult, Box<dyn RollbackGuard>), BootstrapError> {
        let source = ctx_get!(context, RequestedSource);

        let source_path = Path::new(source.as_str());

        if metadata(source_path)?.is_dir() {
            context.put(ResolvedSourceDir(source_path.to_path_buf()));
            return Ok((progress, StageResult::Advance, Box::new(NoRollback)));
        }

        let archive = SourceArchive::sniff(source_path)?;
        let scratch = TempDir::new()?;
        archive.extract(scratch.path())?;

        context.put(ResolvedSourceDir(scratch.path().to_path_buf()));
        context.put(scratch);

        Ok((progress, StageResult::Advance, Box::new(NoRollback)))
    }
}
