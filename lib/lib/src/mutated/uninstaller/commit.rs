// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later

use upac_abi::hook::{CancelToken, ProgressEventBuilder};

use crate::mutated::uninstaller::UninstallError;
use crate::orchestrator::Context;
use crate::orchestrator::stage::{RollbackGuard, Stage};

pub struct CommitStage;

impl Stage<UninstallError> for CommitStage {
    fn run(
        &self, _context: &mut Context, _cancel: &CancelToken, _progress: ProgressEventBuilder,
    ) -> Result<(ProgressEventBuilder, Box<dyn RollbackGuard>), UninstallError> {
        todo!()
    }
}
