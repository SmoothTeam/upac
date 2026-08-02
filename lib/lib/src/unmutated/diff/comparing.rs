use upac_abi::hook::{CancelToken, ProgressEventBuilder};

use crate::orchestrator::Context;
use crate::orchestrator::stage::{RollbackGuard, Stage};
use crate::unmutated::diff::DiffError;

pub struct ComparingStage;

impl Stage<DiffError> for ComparingStage {
    fn run(
        &self, _context: &mut Context, _cancel: &CancelToken, _progress: ProgressEventBuilder,
    ) -> Result<(ProgressEventBuilder, Box<dyn RollbackGuard>), DiffError> {
        todo!()
    }
}
