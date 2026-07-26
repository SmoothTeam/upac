use crate::uninstaller::UninstallError;
use crate::orchestrator::{Context, RollbackStack, Stage};
use crate::types::states::UninstallStateId;

pub struct PreparationStage;

impl Stage<UninstallError> for PreparationStage {
    fn event_id(&self) -> u32 {
        UninstallStateId::Preparation as u32
    }

    fn run(&self, context: &mut Context, stack: &mut RollbackStack) -> Result<(), UninstallError> {
        todo!()
    }
}
