// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_types::error::ErrorKind;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PipelineError {
    Cancelled,
    StagePanicked,
    MissingResult,
    PipelineInvalid,
    RollbackFailed,
}

impl From<PipelineError> for ErrorKind {
    fn from(error: PipelineError) -> Self {
        match error {
            PipelineError::Cancelled => ErrorKind::Cancelled,
            PipelineError::RollbackFailed => ErrorKind::RollbackFailed,
            PipelineError::StagePanicked | PipelineError::MissingResult | PipelineError::PipelineInvalid => {
                ErrorKind::Unexpected
            }
        }
    }
}
