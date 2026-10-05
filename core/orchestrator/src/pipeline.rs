// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::any::{Any, TypeId};

use super::context::Context;
use super::stage::Stage;

#[macro_export]
macro_rules! stages {
    (@steps [$($steps:expr,)*]) => {
        vec![$($steps),*]
    };
    (@steps [$($steps:expr,)*] each::<$item:ty>($($body:expr),+ $(,)?) $(, $($rest:tt)*)?) => {
        $crate::stages!(@steps [$($steps,)* $crate::pipeline::Step::each::<$item>(vec![$(Box::new($body)),+]),] $($($rest)*)?)
    };
    (@steps [$($steps:expr,)*] $stage:expr $(, $($rest:tt)*)?) => {
        $crate::stages!(@steps [$($steps,)* $crate::pipeline::Step::once(Box::new($stage)),] $($($rest)*)?)
    };
    [$($input:tt)+] => {
        $crate::stages!(@steps [] $($input)+)
    };
}

type TakeItemsFn = fn(&mut Context) -> Option<Vec<Box<dyn Any>>>;

pub(crate) struct EachStep<E> {
    pub(crate) item_type: TypeId,
    pub(crate) items_type: TypeId,
    pub(crate) take_items: TakeItemsFn,
    pub(crate) body: Vec<Box<dyn Stage<E>>>,
}

pub(crate) enum StepKind<E> {
    Once(Box<dyn Stage<E>>),
    Each(EachStep<E>),
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

    pub(crate) fn stage_count(&self) -> usize {
        match &self.kind {
            StepKind::Once(_) => 1,
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
