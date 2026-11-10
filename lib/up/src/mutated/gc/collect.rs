// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_types::CancelToken;
use upac_types::error::ErrorKind;

use upac_macro::stage;

use upac_orchestrator::context::Context;
use upac_orchestrator::stage::Stage;

use super::{GcRoots, RetainedPrefixes};

pub struct CollectRootsStage;

#[stage]
impl Stage<ErrorKind> for CollectRootsStage {
    fn run(
        &self, context: &mut Context, _cancel: &CancelToken, _progress: &dyn Fn(Option<&str>, u64, u64),
    ) -> Result<(), ErrorKind> {
        let retained = context.take::<RetainedPrefixes>()?;

        let roots = retained
            .0
            .iter()
            .flat_map(|prefix| prefix.referenced_trees())
            .cloned()
            .collect();

        context.put(GcRoots(roots));

        Ok(())
    }
}
