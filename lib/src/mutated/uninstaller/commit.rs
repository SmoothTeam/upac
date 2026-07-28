use crate::mutated::uninstaller::UninstallError;
use crate::orchestrator::{Context, RollbackStack, Stage};
use crate::types::states::UninstallStateId;

pub struct CommitStage;

impl Stage<UninstallError> for CommitStage {
    fn event_id(&self) -> u32 {
        UninstallStateId::Commit as u32
    }

    fn run(&self, context: &mut Context, stack: &mut RollbackStack) -> Result<(), UninstallError> {
        todo!()
    }
}
