// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later

use std::path::Path;

use upac::errors::CommonError;
use upac::orchestrator::Context;
use upac::orchestrator::stage::{NoRollback, RollbackGuard, Stage};

use upac_abi::hook::{CancelToken, ProgressEventBuilder};

use crate::error::SetupError;
use crate::genesis::GenesisInput;
use crate::meta::SourceDir;

pub struct ReadMetaStage;

impl Stage<SetupError> for ReadMetaStage {
    fn run(
        &self, context: &mut Context, _cancel: &CancelToken, progress: ProgressEventBuilder,
    ) -> Result<(ProgressEventBuilder, Box<dyn RollbackGuard>), SetupError> {
        let input = context.get::<GenesisInput>().ok_or(CommonError::MissingResult)?;

        let source = SourceDir {
            path: Path::new(&input.source_dir),
        };

        let mut meta = source.read(input.meta_filename.as_deref())?;
        let (sha256, installed_size) = source.checksum(!input.empty_config)?;
        meta.sha256 = sha256;
        meta.installed_size = installed_size;

        context.put(meta);

        Ok((progress, Box::new(NoRollback)))
    }
}
