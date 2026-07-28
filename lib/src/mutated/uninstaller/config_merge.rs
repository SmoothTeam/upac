use crate::mutated::uninstaller::UninstallError;
use crate::orchestrator::{Context, RollbackStack, Stage};
use crate::types::states::UninstallStateId;

pub struct ConfigMergeStage;

impl Stage<UninstallError> for ConfigMergeStage {
    fn event_id(&self) -> u32 {
        UninstallStateId::ConfigMerge as u32
    }

    fn run(&self, context: &mut Context, stack: &mut RollbackStack) -> Result<(), UninstallError> {
        todo!()
    }
}
