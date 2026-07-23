use crate::types::errors::UninstallError;
use crate::types::machine::{Context, RollbackStack, Stage};
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
