use upac_abi::hook::{CancelToken, ProgressEventBuilder};

use crate::mutated::update::UpdateError;
use crate::orchestrator::Context;
use crate::orchestrator::stage::{RollbackGuard, Stage};

pub struct CheckoutStage;

impl Stage<UpdateError> for CheckoutStage {
    fn run(
        &self, _context: &mut Context, _cancel: &CancelToken, _progress: ProgressEventBuilder,
    ) -> Result<(ProgressEventBuilder, Box<dyn RollbackGuard>), UpdateError> {
        todo!()
    }
}
