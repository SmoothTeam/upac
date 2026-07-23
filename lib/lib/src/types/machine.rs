use std::any::{Any, TypeId};
use std::collections::{HashMap, HashSet};
use std::ptr::null;

use crate::types::errors::CommonError;
use crate::types::hooks::{HookCancelHandle, HookMessageHandle};

pub struct Context {
    slots: HashMap<TypeId, Box<dyn Any>>,
}

impl Context {
    pub fn new() -> Self {
        Self { slots: HashMap::new() }
    }

    pub fn put<T: Any>(&mut self, value: T) {
        self.slots.insert(TypeId::of::<T>(), Box::new(value));
    }

    pub fn get<T: Any>(&self) -> Option<&T> {
        self.slots
            .get(&TypeId::of::<T>())
            .and_then(|slot| slot.downcast_ref::<T>())
    }

    pub fn take<T: Any>(&mut self) -> Option<T> {
        self.slots
            .remove(&TypeId::of::<T>())
            .and_then(|slot| slot.downcast::<T>().ok())
            .map(|boxed| *boxed)
    }

    fn type_ids(&self) -> HashSet<TypeId> {
        self.slots.keys().copied().collect()
    }
}

pub trait Rollback {
    fn rollback(&mut self);
}

pub struct RollbackStack {
    stack: Vec<Box<dyn Rollback>>,
}

impl RollbackStack {
    fn new() -> Self {
        Self { stack: Vec::new() }
    }

    pub fn push(&mut self, guard: impl Rollback + 'static) {
        self.stack.push(Box::new(guard));
    }

    fn unwind(&mut self) {
        while let Some(mut state) = self.stack.pop() {
            state.rollback();
        }
    }
}

pub trait Stage<E> {
    fn event_id(&self) -> u32 {
        0
    }

    fn requires(&self) -> Vec<TypeId> {
        Vec::new()
    }

    fn provides(&self) -> Vec<TypeId> {
        Vec::new()
    }

    fn run(&self, operation_context: &mut Context, operation_stack: &mut RollbackStack) -> Result<(), E>;
}

pub struct StagePipelineError {
    pub stage_index: usize,
    pub missing: TypeId,
}

pub struct Orchestrator<E> {
    stages: Vec<Box<dyn Stage<E>>>,
}

impl<E> Orchestrator<E> {
    pub fn new(stages: Vec<Box<dyn Stage<E>>>) -> Self {
        Self { stages }
    }

    pub fn validate(&self, operation_context: &Context) -> Result<(), StagePipelineError> {
        let mut available = operation_context.type_ids();

        for (index, stage) in self.stages.iter().enumerate() {
            for required_stage in stage.requires() {
                if !available.contains(&required_stage) {
                    return Err(StagePipelineError {
                        stage_index: index,
                        missing: required_stage,
                    });
                }
            }

            available.extend(stage.provides());
        }

        Ok(())
    }
}

impl<E: From<CommonError>> Orchestrator<E> {
    pub fn run(&self, operation_context: &mut Context) -> Result<(), E> {
        let mut operation_stack = RollbackStack::new();

        for stage in &self.stages {
            if let Some(hook) = operation_context.get::<HookMessageHandle>() {
                hook.call(stage.event_id(), null());
            }

            if let Some(cancel) = operation_context.get::<HookCancelHandle>() {
                if cancel.is_cancelled() {
                    return Err(CommonError::Cancelled.into());
                }
            }

            if let Err(error) = stage.run(operation_context, &mut operation_stack) {
                operation_stack.unwind();
                return Err(error);
            }
        }

        Ok(())
    }
}
