// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 JustPav
//
// SPDX-License-Identifier: LGPL-3.0-or-later

use upac_abi::hook::{CancelToken, ProgressEventBuilder};

use upac_types::TmpPath;

use crate::errors::CommonError;
use crate::mutated::update::UpdateError;
use crate::orchestrator::Context;
use crate::orchestrator::stage::{NoRollback, RollbackGuard, Stage, StageResult};
use crate::plugin::decoder::unpack::PackageUnpacker;

pub struct PreparationStage;

impl Stage<UpdateError> for PreparationStage {
    fn run(
        &self, context: &mut Context, cancel: &CancelToken, progress: ProgressEventBuilder,
    ) -> Result<(ProgressEventBuilder, Box<dyn RollbackGuard>), UpdateError> {
        let package_paths = context.take::<Vec<String>>().ok_or(CommonError::MissingResult)?;
        let tmp_path = context.get::<TmpPath>().ok_or(CommonError::MissingResult)?;

        let mut unpacker = PackageUnpacker::new().map_err(CommonError::Decoder)?;
        let packages = unpacker
            .unpack_all(&package_paths, tmp_path.as_ref(), cancel)
            .map_err(CommonError::Decoder)?;
        context.put(packages);

        Ok((progress, Box::new(NoRollback::new_none(StageResult::Advance))))
    }
}
