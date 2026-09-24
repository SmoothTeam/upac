// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::collections::VecDeque;
use std::fs::read_dir;

use composefs::generic_tree::Stat;
use composefs::repository::ImportContext;
use composefs::tree::FileSystem;

use tempfile::TempDir;

use upac::database::{InMemory, MemoryDatabase};
use upac::errors::CommonError;
use upac::orchestrator::context::{Context, ctx_get};
use upac::orchestrator::stage::{NoRollback, RollbackGuard, Stage, StageResult};
use upac::plugin::decoder::unpack::PackageUnpacker;

use upac_abi::hook::CancelToken;

use upac_types::TmpPath;
use upac_types::hook::ProgressEventBuilder;

use super::error::BootstrapError;
use super::{ConfigState, PrefixTree, ResolvedSourceDir, SetupProgress, UnpackState};

#[cfg(test)]
#[path = "../../../tests/inline/enumerate.rs"]
mod tests;

pub struct EnumeratePackagesStage;

impl Stage<BootstrapError> for EnumeratePackagesStage {
    fn run(
        &self, context: &mut Context, _cancel: &CancelToken, progress: ProgressEventBuilder,
    ) -> Result<(ProgressEventBuilder, StageResult, Box<dyn RollbackGuard>), BootstrapError> {
        let resolved = ctx_get!(context, ResolvedSourceDir);

        let mut package_paths = Vec::new();
        for entry in read_dir(&**resolved)? {
            let entry = entry?;

            if entry.metadata()?.is_file() {
                package_paths.push(entry.path().to_string_lossy().into_owned());
            }
        }

        let total = package_paths.len() as u64;

        let scratch = TempDir::new()?;
        let tmp_path = TmpPath(scratch.path().to_string_lossy().into_owned());

        context.put(SetupProgress {
            pending: VecDeque::new(),
            total,
        });
        context.put(UnpackState {
            unpacker: PackageUnpacker::new().map_err(CommonError::Decoder)?,
            pending_paths: VecDeque::from(package_paths),
        });
        context.put(ConfigState {
            config_tree: FileSystem::new(Stat::uninitialized()),
            database: MemoryDatabase::new_in_memory()?,
        });
        context.put(tmp_path);
        context.put(scratch);
        context.put(ImportContext::default());
        context.put(PrefixTree(FileSystem::new(Stat::uninitialized())));

        Ok((progress, StageResult::Advance, Box::new(NoRollback)))
    }
}
