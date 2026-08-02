use std::any::{Any, TypeId};
use std::collections::{HashMap, HashSet};

use upac_abi::hook::{CancelToken, HookAck, MessageHook, ProgressEventBuilder};

use crate::orchestrator::stage::{RollbackGuard, Stage, StageResult};
use crate::types::errors::CommonError;
use crate::types::lock::{Lock, LockError};

pub mod stage;

pub type StagePipelineError = TypeId;

pub enum OrchestratorError<E> {
    Setup(LockError),
    Stage(usize, E),
}

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

pub struct Orchestrator<E> {
    stages: Vec<Box<dyn Stage<E>>>,
    rollback: Vec<Box<dyn RollbackGuard>>,
}

impl<E: 'static> Orchestrator<E> {
    pub fn new(stages: Vec<Box<dyn Stage<E>>>) -> Self {
        Self {
            stages,
            rollback: Vec::new(),
        }
    }

    pub fn validate(&self, context: &Context) -> Result<(), StagePipelineError> {
        let mut available = context.type_ids();

        for stage in &self.stages {
            for required_stage in stage.requires() {
                if !available.contains(&required_stage) {
                    return Err(required_stage);
                }
            }

            available.extend(stage.provides());
        }

        Ok(())
    }

    fn unwind(&mut self) {
        while let Some(mut guard) = self.rollback.pop() {
            let _ = guard.rollback();
        }
    }
}

impl<E: From<CommonError> + 'static> Orchestrator<E> {
    pub fn run_exclusive(&mut self, context: &mut Context, cancel: &CancelToken) -> Result<(), OrchestratorError<E>> {
        let _lock = Lock::acquire().map_err(OrchestratorError::Setup)?;

        self.run_loop(context, cancel)
            .map_err(|(index, error)| OrchestratorError::Stage(index, error))
    }

    pub fn run_concurrent(&mut self, context: &mut Context, cancel: &CancelToken) -> Result<(), (usize, E)> {
        self.run_loop(context, cancel)
    }

    fn run_loop(&mut self, context: &mut Context, cancel: &CancelToken) -> Result<(), (usize, E)> {
        let mut index = 0;

        while index < self.stages.len() {
            if cancel.is_cancelled() {
                self.unwind();
                return Err((index, CommonError::Cancelled.into()));
            }

            index = self.run_stage(index, context, cancel)?;
        }

        Ok(())
    }

    fn run_stage(&mut self, index: usize, context: &mut Context, cancel: &CancelToken) -> Result<usize, (usize, E)> {
        let progress = ProgressEventBuilder::new(index as u32);

        let (progress, guard) = match self.stages[index].run(context, cancel, progress) {
            Ok(outcome) => outcome,
            Err(error) => {
                self.unwind();
                return Err((index, error));
            }
        };

        Self::send_progress(context, &progress);

        let result = guard.result();
        self.rollback.push(guard);

        self.advance(index, result)
    }

    fn send_progress(context: &Context, progress: &ProgressEventBuilder) {
        if let Some(hook) = context.get::<Box<dyn MessageHook>>() {
            let event = progress.build();
            while hook.send(&event) == HookAck::Retry {}
        }
    }

    fn advance(&mut self, index: usize, result: StageResult) -> Result<usize, (usize, E)> {
        match result {
            StageResult::Advance => Ok(index + 1),
            StageResult::Repeat => Ok(index),
            StageResult::RepeatBack(target) => {
                match self.stages[..index]
                    .iter()
                    .rposition(|stage| (**stage).type_id() == target)
                {
                    Some(found) => Ok(found),
                    None => {
                        self.unwind();
                        Err((index, CommonError::StageNotFound.into()))
                    }
                }
            }
        }
    }
}
