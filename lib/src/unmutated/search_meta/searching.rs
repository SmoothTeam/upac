use upac_abi::hook::{CancelToken, ProgressEventBuilder};

use crate::orchestrator::Context;
use crate::orchestrator::stage::{RollbackGuard, Stage};
use crate::unmutated::search_meta::SearchMetaError;

pub struct SearchingStage;

impl Stage<SearchMetaError> for SearchingStage {
    fn run(
        &self, _context: &mut Context, _cancel: &CancelToken, _progress: ProgressEventBuilder,
    ) -> Result<(ProgressEventBuilder, Box<dyn RollbackGuard>), SearchMetaError> {
        todo!()
    }
}
