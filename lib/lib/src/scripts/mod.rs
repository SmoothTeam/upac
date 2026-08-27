// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_abi::hook::{CancelToken, ProgressEventBuilder};

use crate::errors::CommonError;
use crate::layout::hooks::{HOOK_EXTENSION, HOOKS_DIR, ROOT_CERT_PATH, SIGNATURE_EXTENSION};
use crate::orchestrator::stage::{ConcurrentStage, RollbackGuard, Stage};
use crate::orchestrator::{Context, Orchestrator, ParallelOrchestrator};
use crate::scripts::error::HookError;
use crate::scripts::load::load_hooks;
use crate::scripts::native::NativeTrigger;
use crate::scripts::primitive::Primitive;

pub mod error;
pub mod file;
pub mod load;
pub mod native;
pub mod primitive;

pub struct HookStage {
    pub trigger: NativeTrigger,
}

impl<E: From<CommonError> + Send + 'static> Stage<E> for HookStage {
    fn run(
        &self, context: &mut Context, cancel: &CancelToken, progress: ProgressEventBuilder,
    ) -> Result<(ProgressEventBuilder, Box<dyn RollbackGuard>), E> {
        let runtime = context.runtime().map_err(HookError::from).map_err(CommonError::from)?;

        let hooks =
            load_hooks(HOOKS_DIR, ROOT_CERT_PATH, HOOK_EXTENSION, SIGNATURE_EXTENSION).map_err(CommonError::from)?;

        let matched: Vec<Box<dyn ConcurrentStage<E>>> = hooks
            .into_iter()
            .filter(|hook_file| hook_file.native_trigger() == Some(self.trigger))
            .map(|hook_file| Box::new(hook_file) as Box<dyn ConcurrentStage<E>>)
            .collect();

        ParallelOrchestrator::new(matched, runtime)
            .run_concurrent(context, cancel)
            .map_err(|(_, error)| error)?;

        Ok((progress, Box::new(Vec::<Primitive>::new())))
    }
}
