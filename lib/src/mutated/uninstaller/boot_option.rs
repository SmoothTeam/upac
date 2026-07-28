use crate::mutated::uninstaller::UninstallError;
use crate::orchestrator::{Context, RollbackStack, Stage};
use crate::types::states::UninstallStateId;

pub struct BootOptionStage;

impl Stage<UninstallError> for BootOptionStage {
    fn event_id(&self) -> u32 {
        UninstallStateId::BootOption as u32
    }

    fn run(&self, context: &mut Context, stack: &mut RollbackStack) -> Result<(), UninstallError> {
        todo!()
    }
}
