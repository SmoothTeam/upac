use upac_abi::hook::{CancelToken, ProgressEventBuilder};

use crate::orchestrator::Context;
use crate::orchestrator::stage::{RollbackGuard, Stage};
use crate::unmutated::list_history::ListHistoryError;

pub struct FetchingStage;

impl Stage<ListHistoryError> for FetchingStage {
    fn run(
        &self, _context: &mut Context, _cancel: &CancelToken, _progress: ProgressEventBuilder,
    ) -> Result<(ProgressEventBuilder, Box<dyn RollbackGuard>), ListHistoryError> {
        todo!()
    }
}
