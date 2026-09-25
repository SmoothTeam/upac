// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_abi::hook::CancelToken;

use upac_types::hook::ProgressEventBuilder;

use upac_boot_loader::BootPlugins;
use upac_boot_loader::entry::write_boot_entry;

use upac_composefs::repository::object_id_from_hex;

use upac_deploy::{Deploy, find_esp_mount};

use upac_orchestrator::context::{Context, ctx_get};
use upac_orchestrator::stage::{NoRollback, RollbackGuard, Stage, StageResult};

use super::{RequestedBootPlugin, ResolvedBootEntry, RollbackError, TargetPrefixDigest};

pub struct CheckoutStage;

impl Stage<RollbackError> for CheckoutStage {
    fn run(
        &self, context: &mut Context, _cancel: &CancelToken, progress: ProgressEventBuilder,
    ) -> Result<(ProgressEventBuilder, StageResult, Box<dyn RollbackGuard>), RollbackError> {
        let target = ctx_get!(context, TargetPrefixDigest);
        let deploy = ctx_get!(context, Deploy);
        let requested_boot_plugins = ctx_get!(context, RequestedBootPlugin);

        let repository = deploy.open_repository()?;
        let tree = deploy.open_tree(target)?;
        let digest = object_id_from_hex(target)?;

        let plugin = BootPlugins::new()?.load(requested_boot_plugins)?;

        let esp_mount = find_esp_mount()?;
        let written = write_boot_entry(
            &repository,
            &tree,
            digest,
            &esp_mount,
            target,
            plugin.boot_resource_kind(),
        )?;

        context.put(ResolvedBootEntry {
            plugin,
            entry_name: written.into_entry_name(),
        });

        Ok((progress, StageResult::Advance, Box::new(NoRollback)))
    }
}
