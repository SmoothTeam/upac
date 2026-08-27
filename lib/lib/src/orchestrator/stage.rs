// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::any::{Any, TypeId};

use upac_abi::error::ErrorKind;
use upac_abi::hook::{CancelToken, ProgressEventBuilder};

use crate::orchestrator::Context;

#[derive(Clone, Copy)]
pub enum StageResult {
    Advance,
    Repeat,
    RepeatBack(TypeId),
}

pub trait RollbackGuard: Send + 'static {
    fn new_none(result: StageResult) -> Self
    where
        Self: Sized;

    fn rollback(&mut self) -> Result<(), ErrorKind>;

    fn result(&self) -> StageResult;
}

pub trait RollbackGuardNew: RollbackGuard {
    type Data;

    fn new(data: Self::Data, result: StageResult) -> Self
    where
        Self: Sized;
}

pub trait Stage<E>: Any {
    fn requires(&self) -> Vec<TypeId> {
        Vec::new()
    }

    fn provides(&self) -> Vec<TypeId> {
        Vec::new()
    }

    fn run(
        &self, context: &mut Context, cancel: &CancelToken, progress: ProgressEventBuilder,
    ) -> Result<(ProgressEventBuilder, Box<dyn RollbackGuard>), E>;
}

pub trait ConcurrentStage<E>: Send + 'static {
    fn run(
        self: Box<Self>, progress: ProgressEventBuilder,
    ) -> Result<(ProgressEventBuilder, Box<dyn RollbackGuard>), E>;
}

pub struct NoRollback;

impl RollbackGuard for NoRollback {
    fn new_none(_result: StageResult) -> Self {
        NoRollback
    }

    fn rollback(&mut self) -> Result<(), ErrorKind> {
        Ok(())
    }

    fn result(&self) -> StageResult {
        StageResult::Advance
    }
}
