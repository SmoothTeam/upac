// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 JustPav
//
// SPDX-License-Identifier: LGPL-3.0-or-later

use crate::lock::LockError;

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
