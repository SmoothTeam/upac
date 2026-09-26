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

use crate::mutated::files::{FilesError, NewPrefixDigest, RequestedBootPlugin, ResolvedBootEntry};

pub struct CheckoutStage;

impl Stage<FilesError> for CheckoutStage {
    fn run(
        &self, context: &mut Context, _cancel: &CancelToken, progress: ProgressEventBuilder,
    ) -> Result<(ProgressEventBuilder, StageResult, Box<dyn RollbackGuard>), FilesError> {
        let new_prefix = ctx_get!(context, NewPrefixDigest);

        let deploy = ctx_get!(context, Deploy);

        let requested_boot_plugins = ctx_get!(context, RequestedBootPlugin);

        let repository = deploy.open_repository()?;
        let deploy_tree = deploy.open_tree(new_prefix)?;
        let digest = object_id_from_hex(new_prefix)?;

        let plugin = BootPlugins::new()?.load(requested_boot_plugins)?;

        let esp_mount = find_esp_mount()?;
        let written = write_boot_entry(
            &repository,
            &deploy_tree,
            digest,
            &esp_mount,
            new_prefix,
            plugin.boot_resource_kind()?,
        )?;

        context.put(ResolvedBootEntry {
            plugin,
            entry_name: written.into_entry_name(),
        });

        Ok((progress, StageResult::Advance, Box::new(NoRollback)))
    }
}
