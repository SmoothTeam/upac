// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_types::CancelToken;
use upac_types::error::ErrorKind;

use upac_deploy::Sysroot;

use upac_macro::stage;

use upac_orchestrator::context::Context;
use upac_orchestrator::stage::Stage;

use super::super::prune_prefixes;

pub struct RetentionStage {
    pub retention_depth: usize,
}

#[stage]
impl Stage<ErrorKind> for RetentionStage {
    fn run(
        &self, context: &mut Context, cancel: &CancelToken, progress: &dyn Fn(Option<&str>, u64, u64),
    ) -> Result<(), ErrorKind> {
        prune_prefixes(context.get::<Sysroot>()?, self.retention_depth, cancel, progress)?;

        Ok(())
    }
}
