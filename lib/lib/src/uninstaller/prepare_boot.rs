use crate::types::errors::UninstallError;
use crate::types::machine::{Context, RollbackStack, Stage};
use crate::types::states::UninstallStateId;

pub struct PrepareBootStage;

impl Stage<UninstallError> for PrepareBootStage {
    fn event_id(&self) -> u32 {
        UninstallStateId::PrepareBoot as u32
    }

    fn run(&self, context: &mut Context, stack: &mut RollbackStack) -> Result<(), UninstallError> {
        todo!()
    }
}
