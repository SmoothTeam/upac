// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 JustPav
//
// SPDX-License-Identifier: LGPL-3.0-or-later

use upac_abi::hook::{CancelToken, ProgressEventBuilder};

use crate::orchestrator::Context;
use crate::orchestrator::stage::{RollbackGuard, Stage};
use crate::scripts::error::HookError;
use crate::scripts::file::HookFile;
use crate::scripts::load::load_hooks;
use crate::scripts::native::NativeTrigger;
use crate::types::errors::CommonError;
use crate::types::hooks::{HOOK_EXTENSION, HOOKS_DIR, ROOT_CERT_PATH, SIGNATURE_EXTENSION};

pub mod error;
pub mod file;
pub mod load;
pub mod native;
pub mod primitive;

pub struct HookStage {
    pub trigger: NativeTrigger,
}

impl<E: From<CommonError>> Stage<E> for HookStage {
    fn run(
        &self, context: &mut Context, _cancel: &CancelToken, _progress: ProgressEventBuilder,
    ) -> Result<(ProgressEventBuilder, Box<dyn RollbackGuard>), E> {
        let _runtime = context.runtime().map_err(HookError::from).map_err(CommonError::from)?;

        let hooks =
            load_hooks(HOOKS_DIR, ROOT_CERT_PATH, HOOK_EXTENSION, SIGNATURE_EXTENSION).map_err(CommonError::from)?;

        let _matched: Vec<HookFile> = hooks
            .into_iter()
            .filter(|hook_file| hook_file.native_trigger() == Some(self.trigger))
            .collect();

        todo!()
    }
}
