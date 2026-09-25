// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::collections::HashMap;

use serde::Deserialize;

use upac_types::hook::ProgressEventBuilder;

use upac_orchestrator::stage::{ConcurrentStage, RollbackGuard, StageResult};

use super::error::HookError;
use super::pipeline::{Operation, PipelineTrigger, Timing};
use super::primitive::{ExecutedSteps, Primitive, Step};

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct HookFile {
    #[serde(default)]
    pub(crate) priority: i32,
    #[serde(default)]
    pub(crate) critical: bool,
    pub(crate) operation: Option<Operation>,
    pub(crate) timing: Option<Timing>,
    #[serde(default)]
    pub(crate) triggers: HashMap<String, Vec<String>>,
    #[serde(default)]
    pub(crate) steps: Vec<Primitive>,
}

impl HookFile {
    pub(crate) fn parse(raw: &str) -> Result<Self, HookError> {
        let file: HookFile = toml::from_str(raw)?;

        match (file.operation, file.timing) {
            (Some(_), None) | (None, Some(_)) => return Err(HookError::InvalidTrigger),
            (None, None) if file.triggers.is_empty() => return Err(HookError::NoTrigger),
            _ => {}
        }

        Ok(file)
    }

    pub(crate) fn pipeline_trigger(&self) -> Option<PipelineTrigger> {
        match (self.operation, self.timing) {
            (Some(operation), Some(timing)) => Some(PipelineTrigger { operation, timing }),
            _ => None,
        }
    }
}

impl<E: From<HookError>> ConcurrentStage<E> for HookFile {
    fn run(
        self: Box<Self>, progress: ProgressEventBuilder,
    ) -> Result<(ProgressEventBuilder, StageResult, Box<dyn RollbackGuard>), E> {
        let HookFile { critical, steps, .. } = *self;

        let mut executed = ExecutedSteps(Vec::with_capacity(steps.len()));

        for mut primitive in steps {
            match primitive.execute() {
                Ok(()) => executed.0.push(primitive),
                Err(error) => {
                    if critical {
                        let _ = executed.rollback();

                        return Err(error.into());
                    }

                    break;
                }
            }
        }

        Ok((progress, StageResult::Advance, Box::new(executed)))
    }
}
