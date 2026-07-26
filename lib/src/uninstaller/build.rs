use crate::uninstaller::UninstallError;
use crate::orchestrator::{Context, RollbackStack, Stage};
use crate::types::states::UninstallStateId;

pub struct BuildStage;

impl Stage<UninstallError> for BuildStage {
    fn event_id(&self) -> u32 {
        UninstallStateId::Build as u32
    }

    fn run(&self, context: &mut Context, stack: &mut RollbackStack) -> Result<(), UninstallError> {
        todo!()
    }
}
