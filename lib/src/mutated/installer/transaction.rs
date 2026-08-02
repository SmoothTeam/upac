use upac_abi::hook::{CancelToken, ProgressEventBuilder};

use crate::mutated::installer::InstallError;
use crate::orchestrator::Context;
use crate::orchestrator::stage::{RollbackGuard, Stage};

pub struct TransactionStage;

impl Stage<InstallError> for TransactionStage {
    fn run(
        &self, _context: &mut Context, _cancel: &CancelToken, _progress: ProgressEventBuilder,
    ) -> Result<(ProgressEventBuilder, Box<dyn RollbackGuard>), InstallError> {
        todo!()
    }
}
