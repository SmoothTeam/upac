use crate::types::errors::UninstallError;
use crate::types::machine::{Context, RollbackStack, Stage};

pub struct MergeStage;

impl Stage<UninstallError> for MergeStage {
    fn run(&self, context: &mut Context, stack: &mut RollbackStack) -> Result<(), UninstallError> {
        todo!()
    }
}
