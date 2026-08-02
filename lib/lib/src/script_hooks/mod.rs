// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 JustPav
//
// SPDX-License-Identifier: LGPL-3.0-or-later

use upac_abi::hook::{CancelToken, ProgressEventBuilder};

use crate::orchestrator::Context;
use crate::orchestrator::stage::{RollbackGuard, Stage};

pub enum NativeTrigger {
    PreInstall,
    PostInstall,
    PreUninstall,
    PostUninstall,
    PreUpdate,
    PostUpdate,
    PreRollback,
    PostRollback,
    PreCommit,
    PostCommit,
    PreFiles,
    PostFiles,
}

pub struct HookStage {
    pub trigger: NativeTrigger,
}

impl<E> Stage<E> for HookStage {
    fn run(
        &self, _context: &mut Context, _cancel: &CancelToken, _progress: ProgressEventBuilder,
    ) -> Result<(ProgressEventBuilder, Box<dyn RollbackGuard>), E> {
        todo!()
    }
}
