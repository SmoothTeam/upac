use std::any::{Any, TypeId};
use std::collections::{HashMap, HashSet};

use upac_abi::hook::{CancelToken, HookAck, MessageHook};

use crate::orchestrator::stage::{RollbackGuard, Stage, StageResult};
use crate::types::errors::CommonError;
use crate::types::lock::{Lock, LockError};

pub mod stage;

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

pub struct StagePipelineError(pub TypeId);

pub enum OrchestratorMode {
    Exclusive,
    Concurrent,
}

pub enum RunFailure<E> {
    Setup(LockError),
    Stage(usize, E),
}

pub struct Orchestrator<E> {
    stages: Vec<Box<dyn Stage<E>>>,
    mode: OrchestratorMode,
    rollback: Vec<Box<dyn RollbackGuard>>,
}

impl<E: 'static> Orchestrator<E> {
    pub fn new(stages: Vec<Box<dyn Stage<E>>>, mode: OrchestratorMode) -> Self {
        Self {
            stages,
            mode,
            rollback: Vec::new(),
        }
    }

    pub fn validate(&self, context: &Context) -> Result<(), StagePipelineError> {
        let mut available = context.type_ids();

        for stage in &self.stages {
            for required_stage in stage.requires() {
                if !available.contains(&required_stage) {
                    return Err(StagePipelineError(required_stage));
                }
            }

            available.extend(stage.provides());
        }

        Ok(())
    }

    fn unwind(&mut self) {
        while let Some(mut guard) = self.rollback.pop() {
            guard.rollback();
        }
    }
}

impl<E: From<CommonError> + 'static> Orchestrator<E> {
    pub fn run(&mut self, context: &mut Context, cancel: &CancelToken) -> Result<(), RunFailure<E>> {
        let _lock = match self.mode {
            OrchestratorMode::Exclusive => Some(Lock::acquire().map_err(RunFailure::Setup)?),
            OrchestratorMode::Concurrent => None,
        };

        let mut index = 0;

        while index < self.stages.len() {
            if cancel.is_cancelled() {
                self.unwind();
                return Err(RunFailure::Stage(index, CommonError::Cancelled.into()));
            }

            let outcome = match self.stages[index].run(context, cancel) {
                Ok(outcome) => outcome,
                Err(error) => {
                    self.unwind();
                    return Err(RunFailure::Stage(index, error));
                }
            };

            if let Some(hook) = context.get::<Box<dyn MessageHook>>() {
                let event = outcome.progress.build();
                while hook.send(&event) == HookAck::Retry {}
            }

            if let Some(guard) = outcome.rollback {
                self.rollback.push(guard);
            }

            match outcome.result {
                StageResult::Advance => index += 1,
                StageResult::Repeat => {}
                StageResult::RepeatBack(target) => {
                    match self.stages[..index]
                        .iter()
                        .rposition(|stage| (**stage).type_id() == target)
                    {
                        Some(found) => index = found,
                        None => {
                            self.unwind();
                            return Err(RunFailure::Stage(index, CommonError::StageNotFound.into()));
                        }
                    }
                }
            }
        }

        Ok(())
    }
}
