// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_abi::error::ErrorKind;

use super::lock::LockError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PipelineError {
    Cancelled,
    StageNotFound,
    StagePanicked,
    MissingResult,
    PipelineInvalid,
}

impl From<PipelineError> for ErrorKind {
    fn from(error: PipelineError) -> Self {
        match error {
            PipelineError::Cancelled => ErrorKind::Cancelled,
            PipelineError::StageNotFound
            | PipelineError::StagePanicked
            | PipelineError::MissingResult
            | PipelineError::PipelineInvalid => ErrorKind::Unexpected,
        }
    }
}

pub enum OrchestratorError<E> {
    Setup(LockError),
    Stage(usize, E),
}

impl<E> From<LockError> for OrchestratorError<E> {
    fn from(error: LockError) -> Self {
        OrchestratorError::Setup(error)
    }
}

impl<E> From<(usize, E)> for OrchestratorError<E> {
    fn from(error: (usize, E)) -> Self {
        OrchestratorError::Stage(error.0, error.1)
    }
}
