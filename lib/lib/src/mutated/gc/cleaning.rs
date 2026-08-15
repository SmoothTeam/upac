// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 JustPav
//
// SPDX-License-Identifier: LGPL-3.0-or-later

use upac_abi::hook::{CancelToken, ProgressEventBuilder};

use crate::mutated::gc::GcError;
use crate::orchestrator::Context;
use crate::orchestrator::stage::{RollbackGuard, Stage};

pub struct CleaningStage;

impl Stage<GcError> for CleaningStage {
    fn run(
        &self, _context: &mut Context, _cancel: &CancelToken, _progress: ProgressEventBuilder,
    ) -> Result<(ProgressEventBuilder, Box<dyn RollbackGuard>), GcError> {
        todo!()
    }
}
