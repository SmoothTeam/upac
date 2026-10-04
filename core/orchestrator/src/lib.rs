// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::any::{Any, TypeId};
use std::collections::HashSet;

use upac_types::CancelToken;
use upac_types::error::ErrorKind;
use upac_types::traits::CommandState;

use self::context::Context;
use self::error::PipelineError;
use self::lock::{Lock, LockError};
use self::orchestrator::Orchestrator;
use self::pipeline::{EachStep, ParallelRunner, Step, StepKind};
use self::progress::ProgressEventBuilder;
use self::stage::Stage;

mod layout {
    include!(concat!(env!("OUT_DIR"), "/layout.rs"));
}
mod orchestrator;

pub mod context;
pub mod error;
pub mod lock;
pub mod pipeline;
pub mod progress;
pub mod stage;

pub type StagePipelineError = TypeId;

enum StartedStage<'run, E> {
    Sequential(&'run dyn Stage<E>),
    Parallel(&'run dyn ParallelRunner<E>),
}

impl<E> StartedStage<'_, E> {
    fn rollback(&self, context: &mut Context) -> Result<(), ErrorKind> {
        match self {
            StartedStage::Sequential(stage) => stage.rollback(context),
            StartedStage::Parallel(runner) => runner.rollback(),
        }
    }
}

pub trait OrchestratorRun<E: From<PipelineError> + 'static>: Orchestrator<E> + Sized {
    fn run_mutating<S: CommandState>(self, context: &mut Context, cancel: &CancelToken) -> Result<(), (S, E)>
    where
        E: From<LockError>,
    {
        self.validate(context)
            .map_err(|_| (S::VALIDATION, E::from(PipelineError::PipelineInvalid)))?;

        let _lock = Lock::acquire().map_err(|lock_error| (S::VALIDATION, E::from(lock_error)))?;

        self.execute(context, cancel)
            .map_err(|(index, error)| (S::from_stage_index(index), error))
    }

    fn run_unmutated<R: Any, S: CommandState>(self, context: &mut Context, cancel: &CancelToken) -> Result<R, (S, E)> {
        let available = self
            .validate(context)
            .map_err(|_| (S::VALIDATION, E::from(PipelineError::PipelineInvalid)))?;

        if !available.contains(&TypeId::of::<R>()) {
            return Err((S::VALIDATION, PipelineError::PipelineInvalid.into()));
        }

        self.execute(context, cancel)
            .map_err(|(index, error)| (S::from_stage_index(index), error))?;

        context.take::<R>().map_err(|error| (S::VALIDATION, error.into()))
    }
}

impl<E: From<PipelineError> + 'static, O: Orchestrator<E>> OrchestratorRun<E> for O {}

pub struct SequentialOrchestrator<E> {
    steps: Vec<Step<E>>,
}

impl<E: 'static> SequentialOrchestrator<E> {
    pub fn new(steps: Vec<Step<E>>) -> Self {
        Self { steps }
    }
}

impl<E: From<PipelineError> + 'static> Orchestrator<E> for SequentialOrchestrator<E> {
    fn validate(&self, context: &Context) -> Result<HashSet<TypeId>, StagePipelineError> {
        let mut available = context.type_ids();

        for step in &self.steps {
            match &step.kind {
                StepKind::Once(stage) => Self::validate_stage(stage.as_ref(), &mut available)?,
                StepKind::Each(each) => {
                    if !available.remove(&each.items_type) {
                        return Err(each.items_type);
                    }

                    available.insert(each.item_type);

                    for stage in &each.body {
                        Self::validate_stage(stage.as_ref(), &mut available)?;
                    }

                    available.remove(&each.item_type);
                }
                StepKind::Parallel(parallel) => {
                    if !available.remove(&parallel.items_type) {
                        return Err(parallel.items_type);
                    }
                }
            }
        }

        Ok(available)
    }

    fn execute(self, context: &mut Context, cancel: &CancelToken) -> Result<(), (usize, E)> {
        let mut started = Vec::new();

        Self::run_steps(&self.steps, context, cancel, &mut started)
            .map_err(|(index, error)| (index, Self::unwind(&started, context, error)))
    }
}

impl<E: From<PipelineError> + 'static> SequentialOrchestrator<E> {
    fn validate_stage(stage: &dyn Stage<E>, available: &mut HashSet<TypeId>) -> Result<(), StagePipelineError> {
        for required in stage.requires() {
            if !available.contains(&required) {
                return Err(required);
            }
        }

        available.extend(stage.provides());

        Ok(())
    }

    fn run_steps<'run>(
        steps: &'run [Step<E>], context: &mut Context, cancel: &CancelToken, started: &mut Vec<StartedStage<'run, E>>,
    ) -> Result<(), (usize, E)> {
        let mut index = 0;

        for step in steps {
            match &step.kind {
                StepKind::Once(stage) => {
                    Self::check_cancel(cancel, index)?;
                    started.push(StartedStage::Sequential(stage.as_ref()));

                    let before = ProgressEventBuilder::new(index as u32);
                    let progress = ProgressEventBuilder::new(index as u32);

                    Self::run_stage(stage.as_ref(), context, cancel, before, progress)
                        .map_err(|error| (index, error))?;
                }
                StepKind::Each(each) => Self::run_each(each, index, context, cancel, started)?,
                StepKind::Parallel(parallel) => {
                    Self::check_cancel(cancel, index)?;
                    started.push(StartedStage::Parallel(parallel.runner.as_ref()));

                    context.send_progress(&ProgressEventBuilder::new(index as u32));
                    parallel
                        .runner
                        .run_all(context, cancel, index)
                        .map_err(|error| (index, error))?;
                }
            }

            index += step.stage_count();
        }

        Ok(())
    }

    fn run_each<'run>(
        each: &'run EachStep<E>, first_index: usize, context: &mut Context, cancel: &CancelToken,
        started: &mut Vec<StartedStage<'run, E>>,
    ) -> Result<(), (usize, E)> {
        let items = (each.take_items)(context).unwrap_or_default();
        let total = items.len() as u64;
        let mut body_started = vec![false; each.body.len()];

        for (position, item) in items.into_iter().enumerate() {
            context.put_boxed(each.item_type, item);

            for (offset, stage) in each.body.iter().enumerate() {
                let index = first_index + offset;
                Self::check_cancel(cancel, index)?;

                if !body_started[offset] {
                    body_started[offset] = true;
                    started.push(StartedStage::Sequential(stage.as_ref()));
                }

                let before = ProgressEventBuilder::new(index as u32).progress(position as u64, total);
                let progress = ProgressEventBuilder::new(index as u32).progress(position as u64 + 1, total);

                Self::run_stage(stage.as_ref(), context, cancel, before, progress).map_err(|error| (index, error))?;
            }

            context.discard(each.item_type);
        }

        Ok(())
    }

    fn run_stage(
        stage: &dyn Stage<E>, context: &mut Context, cancel: &CancelToken, before: ProgressEventBuilder,
        progress: ProgressEventBuilder,
    ) -> Result<(), E> {
        context.send_progress(&before);

        let progress = stage.run(context, cancel, progress)?;
        context.send_progress(&progress);

        Ok(())
    }

    fn check_cancel(cancel: &CancelToken, index: usize) -> Result<(), (usize, E)> {
        if cancel.is_cancelled() {
            return Err((index, PipelineError::Cancelled.into()));
        }

        Ok(())
    }

    fn unwind(started: &[StartedStage<'_, E>], context: &mut Context, error: E) -> E {
        let mut rollback_failed = false;

        for stage in started.iter().rev() {
            if stage.rollback(context).is_err() {
                rollback_failed = true;
            }
        }

        if rollback_failed {
            return PipelineError::RollbackFailed.into();
        }

        error
    }
}
