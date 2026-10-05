// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::any::TypeId;
use std::collections::HashSet;

use upac_types::CancelToken;
use upac_types::progress::ProgressEvent;

use super::StagePipelineError;
use super::context::Context;

pub trait Orchestrator<E> {
    fn validate(&self, context: &Context) -> Result<HashSet<TypeId>, StagePipelineError>;

    fn execute(
        self, context: &mut Context, cancel: &CancelToken, on_progress: &dyn Fn(&ProgressEvent),
    ) -> Result<(), (usize, E)>;
}
