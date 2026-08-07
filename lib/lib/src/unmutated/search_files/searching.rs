// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 JustPav
//
// SPDX-License-Identifier: LGPL-3.0-or-later

use upac_abi::hook::{CancelToken, ProgressEventBuilder};

use crate::orchestrator::Context;
use crate::orchestrator::stage::{RollbackGuard, Stage};
use crate::unmutated::search_files::SearchFilesError;

pub struct SearchingStage;

impl Stage<SearchFilesError> for SearchingStage {
    fn run(
        &self, _context: &mut Context, _cancel: &CancelToken, _progress: ProgressEventBuilder,
    ) -> Result<(ProgressEventBuilder, Box<dyn RollbackGuard>), SearchFilesError> {
        todo!()
    }
}
