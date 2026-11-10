// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_types::CancelToken;
use upac_types::error::ErrorKind;
use upac_types::response::entry::HistoryEntry;
use upac_types::response::unmutated::ListHistoryResponse;

use upac_deploy::Sysroot;
use upac_deploy::deployment::Deployment;

use upac_macro::stage;

use upac_orchestrator::context::Context;
use upac_orchestrator::stage::Stage;

use super::super::config_commit_entry;

pub struct FetchingStage;

#[stage]
impl Stage<ErrorKind> for FetchingStage {
    fn run(
        &self, context: &mut Context, _cancel: &CancelToken, _progress: &dyn Fn(Option<&str>, u64, u64),
    ) -> Result<(), ErrorKind> {
        let history = context
            .get::<Sysroot>()?
            .prefixes()?
            .iter()
            .map(|prefix| HistoryEntry {
                prefix_digest: prefix.digest().to_hex(),
                subject: prefix.subject().to_owned(),
                message: prefix.message().map(str::to_owned),
                timestamp: prefix.timestamp(),
                working_config: prefix.current_config().map(|config| config.digest().to_hex()),
                config_history: prefix.configs().iter().map(config_commit_entry).collect(),
            })
            .collect();

        context.put(ListHistoryResponse { history });

        Ok(())
    }
}
