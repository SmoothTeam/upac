use crate::types::errors::UninstallError;
use crate::types::machine::{Context, RollbackStack, Stage};
use crate::types::states::UninstallStateId;

pub struct TransactionStage;

impl Stage<UninstallError> for TransactionStage {
    fn event_id(&self) -> u32 {
        UninstallStateId::Transaction as u32
    }

    fn run(&self, context: &mut Context, stack: &mut RollbackStack) -> Result<(), UninstallError> {
        todo!()
    }
}
