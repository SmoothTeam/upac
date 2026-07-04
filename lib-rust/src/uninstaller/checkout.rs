use crate::types::errors::UninstallError;
use crate::types::machine::{Context, RollbackStack, Stage};

pub struct CheckoutStage;

impl Stage<UninstallError> for CheckoutStage {
    fn run(&self, context: &mut Context, stack: &mut RollbackStack) -> Result<(), UninstallError> {
        todo!()
    }
}
