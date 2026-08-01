use upac_abi::hook::CancelToken;

use crate::mutated::uninstaller::UninstallError;
use crate::orchestrator::Context;
use crate::orchestrator::stage::{Stage, StageOutcome};

pub struct PrepareBootStage;

impl Stage<UninstallError> for PrepareBootStage {
    fn run(&self, context: &mut Context, cancel: &CancelToken) -> Result<StageOutcome, UninstallError> {
        todo!()
    }
}
