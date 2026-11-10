// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_types::CancelToken;
use upac_types::error::ErrorKind;
use upac_types::response::unmutated::ListConfigResponse;

use upac_deploy::Sysroot;

use upac_macro::stage;

use upac_orchestrator::context::Context;
use upac_orchestrator::stage::Stage;

use super::super::{config_commit_entry, requested_prefix};
use super::RequestedPrefixDigest;

pub struct FetchingStage;

#[stage]
impl Stage<ErrorKind> for FetchingStage {
    fn run(
        &self, context: &mut Context, _cancel: &CancelToken, _progress: &dyn Fn(Option<&str>, u64, u64),
    ) -> Result<(), ErrorKind> {
        let prefix = requested_prefix(
            context.get::<Sysroot>()?,
            context.get::<RequestedPrefixDigest>()?.0.as_ref(),
        )?;

        let commits = prefix.configs().iter().map(config_commit_entry).collect();

        context.put(ListConfigResponse { commits });

        Ok(())
    }
}
