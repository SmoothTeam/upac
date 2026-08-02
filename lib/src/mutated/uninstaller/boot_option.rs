use upac_abi::hook::{CancelToken, ProgressEventBuilder};

use crate::mutated::uninstaller::UninstallError;
use crate::orchestrator::Context;
use crate::orchestrator::stage::{RollbackGuard, Stage};

pub struct BootOptionStage;

impl Stage<UninstallError> for BootOptionStage {
    fn run(
        &self,
        context: &mut Context,
        cancel: &CancelToken,
    ) -> Result<(ProgressEventBuilder, Box<dyn RollbackGuard>), UninstallError> {
        todo!()
    }
}
