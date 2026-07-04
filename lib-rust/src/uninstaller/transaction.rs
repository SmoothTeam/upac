use crate::types::errors::UninstallError;
use crate::types::machine::{Context, RollbackStack, Stage};

pub struct TransactionStage;

impl Stage<UninstallError> for TransactionStage {
    fn run(&self, context: &mut Context, stack: &mut RollbackStack) -> Result<(), UninstallError> {
        todo!()
    }
}
