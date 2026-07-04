use crate::types::errors::UninstallError;
use crate::types::machine::{Context, RollbackStack, Stage};

pub struct PreparationStage;

impl Stage<UninstallError> for PreparationStage {
    fn run(&self, context: &mut Context, stack: &mut RollbackStack) -> Result<(), UninstallError> {
        todo!()
    }
}
