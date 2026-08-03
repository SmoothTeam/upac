// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 JustPav
//
// SPDX-License-Identifier: LGPL-3.0-or-later

use std::any::{Any, TypeId};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use tokio::runtime::Runtime;
use tokio::task::JoinSet;
use upac_abi::hook::{CancelToken, HookAck, MessageHook, ProgressEventBuilder};

use crate::orchestrator::cursor::Cursor;
use crate::orchestrator::stage::{ConcurrentStage, RollbackGuard, Stage, StageResult};
use crate::types::errors::CommonError;
use crate::types::lock::{Lock, LockError};

pub mod cursor;
pub mod stage;

pub type StagePipelineError = TypeId;

pub enum OrchestratorError<E> {
    Setup(LockError),
    Stage(usize, E),
}

pub struct Context {
    slots: HashMap<TypeId, Box<dyn Any>>,
    rollback: Vec<Box<dyn RollbackGuard>>,
}

impl Context {
    pub fn new() -> Self {
        Self {
            slots: HashMap::new(),
            rollback: Vec::new(),
        }
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

    fn unwind(&mut self) {
        while let Some(mut guard) = self.rollback.pop() {
            let _ = guard.rollback();
        }
    }
}

pub trait Orchestrator<E>: Sized {
    fn run_exclusive(self, context: &mut Context, cancel: &CancelToken) -> Result<(), OrchestratorError<E>>;

    fn run_concurrent(self, context: &mut Context, cancel: &CancelToken) -> Result<(), (usize, E)>;

    fn send_progress(context: &Context, progress: &ProgressEventBuilder) {
        if let Some(hook) = context.get::<Box<dyn MessageHook>>() {
            let event = progress.build();
            while hook.send(&event) == HookAck::Retry {}
        }
    }
}

pub struct SequentialOrchestrator<E> {
    cursor: Cursor<E>,
}

impl<E: 'static> SequentialOrchestrator<E> {
    pub fn new(stages: Vec<Box<dyn Stage<E>>>) -> Self {
        Self {
            cursor: Cursor::new(stages),
        }
    }

    pub fn validate(&self, context: &Context) -> Result<(), StagePipelineError> {
        let mut available = context.type_ids();

        for stage in self.cursor.stages() {
            for required_stage in stage.requires() {
                if !available.contains(&required_stage) {
                    return Err(required_stage);
                }
            }

            available.extend(stage.provides());
        }

        Ok(())
    }
}

impl<E: From<CommonError> + 'static> Orchestrator<E> for SequentialOrchestrator<E> {
    fn run_exclusive(mut self, context: &mut Context, cancel: &CancelToken) -> Result<(), OrchestratorError<E>> {
        let _lock = Lock::acquire().map_err(OrchestratorError::Setup)?;

        Self::run(&mut self.cursor, context, cancel).map_err(|(index, error)| OrchestratorError::Stage(index, error))
    }

    fn run_concurrent(mut self, context: &mut Context, cancel: &CancelToken) -> Result<(), (usize, E)> {
        Self::run(&mut self.cursor, context, cancel)
    }
}

impl<E: From<CommonError> + 'static> SequentialOrchestrator<E>
where
    Self: Orchestrator<E>,
{
    fn run(cursor: &mut Cursor<E>, context: &mut Context, cancel: &CancelToken) -> Result<(), (usize, E)> {
        let mut previous = None;

        while let Some(index) = cursor.next(context, cancel, previous.take())? {
            previous = Some(Self::run_stage(&cursor.stages()[index], index, context, cancel)?);
        }

        Ok(())
    }

    fn run_stage(
        stage: &Box<dyn Stage<E>>, index: usize, context: &mut Context, cancel: &CancelToken,
    ) -> Result<StageResult, (usize, E)> {
        let progress = ProgressEventBuilder::new(index as u32);

        let (progress, guard) = match stage.run(context, cancel, progress) {
            Ok(outcome) => outcome,
            Err(error) => {
                context.unwind();
                return Err((index, error));
            }
        };

        Self::send_progress(context, &progress);

        let result = guard.result();
        context.rollback.push(guard);

        Ok(result)
    }
}

pub struct ParallelOrchestrator<E> {
    stages: Vec<Box<dyn ConcurrentStage<E>>>,
    runtime: Arc<Runtime>,
}

impl<E: 'static> ParallelOrchestrator<E> {
    pub fn new(stages: Vec<Box<dyn ConcurrentStage<E>>>, runtime: Arc<Runtime>) -> Self {
        Self { stages, runtime }
    }
}

impl<E: From<CommonError> + Send + 'static> Orchestrator<E> for ParallelOrchestrator<E> {
    fn run_exclusive(self, context: &mut Context, cancel: &CancelToken) -> Result<(), OrchestratorError<E>> {
        let _lock = Lock::acquire().map_err(OrchestratorError::Setup)?;

        Self::run_parallel(self.stages, &self.runtime, context, cancel)
            .map_err(|(index, error)| OrchestratorError::Stage(index, error))
    }

    fn run_concurrent(self, context: &mut Context, cancel: &CancelToken) -> Result<(), (usize, E)> {
        Self::run_parallel(self.stages, &self.runtime, context, cancel)
    }
}

impl<E: From<CommonError> + Send + 'static> ParallelOrchestrator<E>
where
    Self: Orchestrator<E>,
{
    fn run_parallel(
        stages: Vec<Box<dyn ConcurrentStage<E>>>, runtime: &Runtime, context: &mut Context, cancel: &CancelToken,
    ) -> Result<(), (usize, E)> {
        if cancel.is_cancelled() {
            return Err((0, CommonError::Cancelled.into()));
        }

        runtime.block_on(Self::run_batch(stages, context))
    }

    async fn run_batch(stages: Vec<Box<dyn ConcurrentStage<E>>>, context: &mut Context) -> Result<(), (usize, E)> {
        let mut set = JoinSet::new();

        for stage in stages {
            set.spawn_blocking(move || stage.run(ProgressEventBuilder::new(0)));
        }

        while let Some(outcome) = set.join_next().await {
            match outcome {
                Ok(Ok((progress, guard))) => {
                    Self::send_progress(context, &progress);
                    context.rollback.push(guard);
                }
                Ok(Err(error)) => {
                    context.unwind();
                    return Err((0, error));
                }
                Err(_) => {
                    context.unwind();
                    return Err((0, CommonError::StagePanicked.into()));
                }
            }
        }

        Ok(())
    }
}
