// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::any::{Any, TypeId};
use std::cell::RefCell;
use std::collections::HashSet;

use upac_types::CancelToken;
use upac_types::progress::ProgressEvent;
use upac_types::traits::CommandState;

use self::context::Context;
use self::error::PipelineError;
use self::lock::{Lock, LockError};
use self::orchestrator::Orchestrator;
use self::pipeline::{EachStep, Step, StepKind};
use self::stage::Stage;

mod layout {
    include!(concat!(env!("OUT_DIR"), "/layout.rs"));
}
mod orchestrator;

pub mod context;
pub mod error;
pub mod lock;
pub mod pipeline;
pub mod stage;

pub type StagePipelineError = TypeId;

pub trait OrchestratorRun<StageError: From<PipelineError> + 'static>: Orchestrator<StageError> + Sized {
    fn run_mutating<State: CommandState>(
        self, context: &mut Context, cancel: &CancelToken, on_progress: &dyn Fn(&ProgressEvent),
    ) -> Result<(), (State, StageError, Option<String>)>
    where
        StageError: From<LockError>,
    {
        self.validate(context).map_err(|_| {
            (
                State::VALIDATION,
                StageError::from(PipelineError::PipelineInvalid),
                None,
            )
        })?;

        let _lock = Lock::acquire().map_err(|lock_error| (State::VALIDATION, StageError::from(lock_error), None))?;

        self.execute(context, cancel, on_progress)
            .map_err(|(index, error, subject)| (State::from_stage_index(index), error, subject))
    }

    fn run_mutating_with_response<Response: Any, State: CommandState>(
        self, context: &mut Context, cancel: &CancelToken, on_progress: &dyn Fn(&ProgressEvent),
    ) -> Result<Response, (State, StageError, Option<String>)>
    where
        StageError: From<LockError>,
    {
        let available = self.validate(context).map_err(|_| {
            (
                State::VALIDATION,
                StageError::from(PipelineError::PipelineInvalid),
                None,
            )
        })?;

        if !available.contains(&TypeId::of::<Response>()) {
            return Err((State::VALIDATION, PipelineError::PipelineInvalid.into(), None));
        }

        let _lock = Lock::acquire().map_err(|lock_error| (State::VALIDATION, StageError::from(lock_error), None))?;

        self.execute(context, cancel, on_progress)
            .map_err(|(index, error, subject)| (State::from_stage_index(index), error, subject))?;

        context
            .take::<Response>()
            .map_err(|error| (State::VALIDATION, error.into(), None))
    }

    fn run_unmutated<Response: Any, State: CommandState>(
        self, context: &mut Context, cancel: &CancelToken, on_progress: &dyn Fn(&ProgressEvent),
    ) -> Result<Response, (State, StageError, Option<String>)> {
        let available = self.validate(context).map_err(|_| {
            (
                State::VALIDATION,
                StageError::from(PipelineError::PipelineInvalid),
                None,
            )
        })?;

        if !available.contains(&TypeId::of::<Response>()) {
            return Err((State::VALIDATION, PipelineError::PipelineInvalid.into(), None));
        }

        self.execute(context, cancel, on_progress)
            .map_err(|(index, error, subject)| (State::from_stage_index(index), error, subject))?;

        context
            .take::<Response>()
            .map_err(|error| (State::VALIDATION, error.into(), None))
    }
}

impl<StageError: From<PipelineError> + 'static, Pipeline: Orchestrator<StageError>> OrchestratorRun<StageError>
    for Pipeline
{
}

pub struct SequentialOrchestrator<StageError> {
    steps: Vec<Step<StageError>>,
}

impl<StageError: 'static> SequentialOrchestrator<StageError> {
    pub fn new(steps: Vec<Step<StageError>>) -> Self {
        Self { steps }
    }
}

impl<StageError: From<PipelineError> + 'static> Orchestrator<StageError> for SequentialOrchestrator<StageError> {
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
            }
        }

        Ok(available)
    }

    fn execute(
        self, context: &mut Context, cancel: &CancelToken, on_progress: &dyn Fn(&ProgressEvent),
    ) -> Result<(), (usize, StageError, Option<String>)> {
        let mut started = Vec::new();
        let last_subject = RefCell::new(None);

        Self::run_steps(&self.steps, context, cancel, on_progress, &last_subject, &mut started)
            .map_err(|(index, error)| (index, Self::unwind(&started, context, error), last_subject.take()))
    }
}

impl<StageError: From<PipelineError> + 'static> SequentialOrchestrator<StageError> {
    fn validate_stage(
        stage: &dyn Stage<StageError>, available: &mut HashSet<TypeId>,
    ) -> Result<(), StagePipelineError> {
        for required in stage.requires() {
            if !available.contains(&required) {
                return Err(required);
            }
        }

        available.extend(stage.provides());

        Ok(())
    }

    fn run_steps<'run>(
        steps: &'run [Step<StageError>], context: &mut Context, cancel: &CancelToken,
        on_progress: &dyn Fn(&ProgressEvent), last_subject: &RefCell<Option<String>>,
        started: &mut Vec<&'run dyn Stage<StageError>>,
    ) -> Result<(), (usize, StageError)> {
        let mut index = 0;

        for step in steps {
            match &step.kind {
                StepKind::Once(stage) => {
                    Self::check_cancel(cancel, index)?;
                    started.push(stage.as_ref());

                    Self::report(on_progress, index, 0, 0);
                    Self::run_stage(stage.as_ref(), index, context, cancel, on_progress, last_subject)
                        .map_err(|error| (index, error))?;
                }
                StepKind::Each(each) => {
                    Self::run_each(each, index, context, cancel, on_progress, last_subject, started)?
                }
            }

            index += step.stage_count();
        }

        Ok(())
    }

    fn run_each<'run>(
        each: &'run EachStep<StageError>, first_index: usize, context: &mut Context, cancel: &CancelToken,
        on_progress: &dyn Fn(&ProgressEvent), last_subject: &RefCell<Option<String>>,
        started: &mut Vec<&'run dyn Stage<StageError>>,
    ) -> Result<(), (usize, StageError)> {
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
                    started.push(stage.as_ref());
                }

                Self::report(on_progress, index, position as u64, total);
                Self::run_stage(stage.as_ref(), index, context, cancel, on_progress, last_subject)
                    .map_err(|error| (index, error))?;
                Self::report(on_progress, index, position as u64 + 1, total);
            }

            context.discard(each.item_type);
        }

        Ok(())
    }

    fn run_stage(
        stage: &dyn Stage<StageError>, index: usize, context: &mut Context, cancel: &CancelToken,
        on_progress: &dyn Fn(&ProgressEvent), last_subject: &RefCell<Option<String>>,
    ) -> Result<(), StageError> {
        last_subject.replace(None);

        let stage_progress = |subject: Option<&str>, current: u64, total: u64| {
            if let Some(subject) = subject {
                last_subject.replace(Some(subject.to_owned()));
            }

            on_progress(&ProgressEvent {
                stage: index as u32,
                subject,
                current,
                total,
            })
        };

        stage.run(context, cancel, &stage_progress)
    }

    fn report(on_progress: &dyn Fn(&ProgressEvent), index: usize, current: u64, total: u64) {
        on_progress(&ProgressEvent {
            stage: index as u32,
            subject: None,
            current,
            total,
        });
    }

    fn check_cancel(cancel: &CancelToken, index: usize) -> Result<(), (usize, StageError)> {
        if cancel.is_cancelled() {
            return Err((index, PipelineError::Cancelled.into()));
        }

        Ok(())
    }

    fn unwind(started: &[&dyn Stage<StageError>], context: &mut Context, error: StageError) -> StageError {
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
