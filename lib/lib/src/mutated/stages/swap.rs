// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_types::CancelToken;
use upac_types::error::ErrorKind;
use upac_types::request::booter::BootPluginSetOneShotRequest;

use upac_macro::stage;

use upac_orchestrator::context::Context;
use upac_orchestrator::stage::Stage;

use super::ResolvedBootEntry;

pub struct SwapStage;

#[stage]
impl Stage<ErrorKind> for SwapStage {
    fn run(
        &self, context: &mut Context, _cancel: &CancelToken, _progress: &dyn Fn(Option<&str>, u64, u64),
    ) -> Result<(), ErrorKind> {
        let resolved = context.take::<ResolvedBootEntry>()?;

        resolved.plugin.set_one_shot(BootPluginSetOneShotRequest {
            entry_name: resolved.entry_name,
        })?;

        Ok(())
    }
}
