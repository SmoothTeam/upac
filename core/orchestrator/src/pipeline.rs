// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::any::{Any, TypeId};
use std::marker::PhantomData;
use std::thread::scope;

use upac_abi::error::ErrorKind;
use upac_abi::hook::CancelToken;

use super::context::Context;
use super::error::PipelineError;
use super::progress::ProgressEventBuilder;
use super::stage::{ParallelStage, Stage};

#[macro_export]
macro_rules! stages {
    (@steps [$($steps:expr,)*]) => {
        vec![$($steps),*]
    };
    (@steps [$($steps:expr,)*] each::<$item:ty>($($body:expr),+ $(,)?) $(, $($rest:tt)*)?) => {
        $crate::stages!(@steps [$($steps,)* $crate::pipeline::Step::each::<$item>(vec![$(Box::new($body)),+]),] $($($rest)*)?)
    };
    (@steps [$($steps:expr,)*] parallel::<$item:ty>($stage:expr $(,)?) $(, $($rest:tt)*)?) => {
        $crate::stages!(@steps [$($steps,)* $crate::pipeline::Step::parallel::<$item, _>($stage),] $($($rest)*)?)
    };
    (@steps [$($steps:expr,)*] $stage:expr $(, $($rest:tt)*)?) => {
        $crate::stages!(@steps [$($steps,)* $crate::pipeline::Step::once(Box::new($stage)),] $($($rest)*)?)
    };
    [$($input:tt)+] => {
        $crate::stages!(@steps [] $($input)+)
    };
}

pub(crate) trait ParallelRunner<E> {
    fn run_all(&self, context: &mut Context, cancel: &CancelToken, index: usize) -> Result<(), E>;

    fn rollback(&self) -> Result<(), ErrorKind>;
}

pub(crate) struct EachStep<E> {
    pub(crate) item_type: TypeId,
    pub(crate) items_type: TypeId,
    pub(crate) take_items: fn(&mut Context) -> Option<Vec<Box<dyn Any>>>,
    pub(crate) body: Vec<Box<dyn Stage<E>>>,
}

pub(crate) struct ParallelStep<E> {
    pub(crate) items_type: TypeId,
    pub(crate) runner: Box<dyn ParallelRunner<E>>,
}

pub(crate) enum StepKind<E> {
    Once(Box<dyn Stage<E>>),
    Each(EachStep<E>),
    Parallel(ParallelStep<E>),
}

pub struct Step<E> {
    pub(crate) kind: StepKind<E>,
}

impl<E: 'static> Step<E> {
    pub fn once(stage: Box<dyn Stage<E>>) -> Self {
        Self {
            kind: StepKind::Once(stage),
        }
    }

    pub fn each<T: Any>(body: Vec<Box<dyn Stage<E>>>) -> Self {
        Self {
            kind: StepKind::Each(EachStep {
                item_type: TypeId::of::<T>(),
                items_type: TypeId::of::<Vec<T>>(),
                take_items: take_boxed_items::<T>,
                body,
            }),
        }
    }

    pub fn parallel<T, S>(stage: S) -> Self
    where
        T: Any + Send,
        S: ParallelStage<E, T> + 'static,
        E: From<PipelineError> + Send,
    {
        Self {
            kind: StepKind::Parallel(ParallelStep {
                items_type: TypeId::of::<Vec<T>>(),
                runner: Box::new(ParallelStageRunner {
                    stage,
                    item: PhantomData,
                }),
            }),
        }
    }

    pub(crate) fn stage_count(&self) -> usize {
        match &self.kind {
            StepKind::Once(_) | StepKind::Parallel(_) => 1,
            StepKind::Each(each) => each.body.len(),
        }
    }
}

fn take_boxed_items<T: Any>(context: &mut Context) -> Option<Vec<Box<dyn Any>>> {
    context
        .take::<Vec<T>>()
        .ok()
        .map(|items| items.into_iter().map(|item| Box::new(item) as Box<dyn Any>).collect())
}

struct ParallelStageRunner<T, S> {
    stage: S,
    item: PhantomData<fn() -> T>,
}

impl<E, T, S> ParallelRunner<E> for ParallelStageRunner<T, S>
where
    T: Any + Send,
    S: ParallelStage<E, T>,
    E: From<PipelineError> + Send,
{
    fn run_all(&self, context: &mut Context, cancel: &CancelToken, index: usize) -> Result<(), E> {
        let items = context.take::<Vec<T>>().unwrap_or_default();
        let total = items.len() as u64;
        let stage = &self.stage;

        let outcomes: Vec<_> = scope(|scope| {
            let handles: Vec<_> = items
                .into_iter()
                .map(|item| scope.spawn(move || stage.run(item, cancel, ProgressEventBuilder::new(index as u32))))
                .collect();

            handles.into_iter().map(|handle| handle.join()).collect()
        });

        let mut processed = 0;
        let mut failure = None;

        for outcome in outcomes {
            match outcome {
                Ok(Ok(progress)) => {
                    processed += 1;
                    context.send_progress(&progress.progress(processed, total));
                }
                Ok(Err(error)) => {
                    if failure.is_none() {
                        failure = Some(error);
                    }
                }
                Err(_) => {
                    if failure.is_none() {
                        failure = Some(PipelineError::StagePanicked.into());
                    }
                }
            }
        }

        match failure {
            Some(error) => Err(error),
            None => Ok(()),
        }
    }

    fn rollback(&self) -> Result<(), ErrorKind> {
        self.stage.rollback()
    }
}
