use upac_abi::hook::{CancelToken, ProgressEventBuilder};

use crate::mutated::rollback::RollbackError;
use crate::orchestrator::Context;
use crate::orchestrator::stage::{RollbackGuard, Stage};

pub struct SwapStage;

impl Stage<RollbackError> for SwapStage {
    fn run(
        &self, _context: &mut Context, _cancel: &CancelToken, _progress: ProgressEventBuilder,
    ) -> Result<(ProgressEventBuilder, Box<dyn RollbackGuard>), RollbackError> {
        todo!()
    }
}
