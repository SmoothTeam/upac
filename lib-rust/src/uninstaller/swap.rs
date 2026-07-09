use crate::types::errors::UninstallError;
use crate::types::machine::{Context, RollbackStack, Stage};
use crate::types::states::UninstallStateId;

pub struct SwapStage;

impl Stage<UninstallError> for SwapStage {
    fn event_id(&self) -> u32 {
        UninstallStateId::Swap as u32
    }

    fn run(&self, context: &mut Context, stack: &mut RollbackStack) -> Result<(), UninstallError> {
        todo!()
    }
}
