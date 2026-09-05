// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::fs::remove_dir_all;
use std::path::PathBuf;

use upac_abi::error::ErrorKind;
use upac_abi::hook::{CancelToken, ProgressEventBuilder};

use upac_types::TmpPath;

use crate::errors::CommonError;
use crate::mutated::update::{PendingPackagePaths, PendingPackages, TotalPackages, UnpackerState, UpdateError};
use crate::orchestrator::stage::{RollbackGuard, Stage, StageResult};
use crate::orchestrator::{Context, ctx_get, ctx_take};

pub struct PreparationStage;

struct UnpackedPackageDir(PathBuf);

impl Stage<UpdateError> for PreparationStage {
    fn run(
        &self, context: &mut Context, cancel: &CancelToken, mut progress: ProgressEventBuilder,
    ) -> Result<(ProgressEventBuilder, StageResult, Box<dyn RollbackGuard>), UpdateError> {
        let mut pending_paths = ctx_take!(context, PendingPackagePaths);
        let mut unpacker = ctx_take!(context, UnpackerState);
        let mut pending_packages = ctx_take!(context, PendingPackages);

        let tmp_path = ctx_get!(context, TmpPath);
        let total_packages = ctx_get!(context, TotalPackages);

        let package_path = pending_paths.0.pop_front().ok_or(CommonError::MissingResult)?;
        let index = pending_packages.0.len();

        let (package, trigger) = unpacker
            .0
            .unpack_one(&package_path, index, tmp_path.as_ref(), cancel)
            .map_err(CommonError::Decoder)?;

        let guard = UnpackedPackageDir(PathBuf::from(&package.temp_package_path));

        pending_packages.0.push_back((package, trigger));

        let remaining = pending_paths.0.len() as u64;
        let processed = total_packages.0 - remaining;
        progress = progress.subject(package_path).progress(processed, total_packages.0);

        let result = if pending_paths.0.is_empty() {
            StageResult::Advance
        } else {
            StageResult::Repeat
        };

        context.put(pending_paths);
        context.put(unpacker);
        context.put(pending_packages);

        Ok((progress, result, Box::new(guard)))
    }
}

impl RollbackGuard for UnpackedPackageDir {
    fn rollback(&mut self) -> Result<(), ErrorKind> {
        let _ = remove_dir_all(&self.0);

        Ok(())
    }
}
