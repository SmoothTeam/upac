// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_types::CancelToken;
use upac_types::error::ErrorKind;
use upac_types::response::bootstrap::SetupBootstrapImportResponse;

use upac_deploy::Sysroot;
use upac_deploy::working::CommittedPrefix;

use upac_macro::stage;

use upac_orchestrator::context::Context;
use upac_orchestrator::stage::Stage;

use super::{EmptyConfig, SourceDir};

use crate::layout::bootstrap::CONFIG_DIR;

pub struct ConfigStage;

#[stage]
impl Stage<ErrorKind> for ConfigStage {
    fn run(
        &self, context: &mut Context, cancel: &CancelToken, _progress: &dyn Fn(Option<&str>, u64, u64),
    ) -> Result<(), ErrorKind> {
        let committed = context.take::<CommittedPrefix>()?;
        let config_dir = context.get::<SourceDir>()?.0.join(CONFIG_DIR);

        let mut config = if context.get::<EmptyConfig>()?.0 {
            context.get::<Sysroot>()?.repo().empty_tree()
        } else {
            committed.defaults
        };

        if config_dir.is_dir() {
            config.import_dir("", &config_dir, cancel, &mut |_| {})?;
        }

        context.put(SetupBootstrapImportResponse {
            prefix_digest: committed.digest.to_hex(),
            config_digest: config.commit()?.to_hex(),
        });

        Ok(())
    }
}
