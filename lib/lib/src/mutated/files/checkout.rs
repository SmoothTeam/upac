// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_abi::hook::{CancelToken, ProgressEventBuilder};

use crate::boot::write_boot_entry;
use crate::composefs::repository::object_id_from_hex;
use crate::deploy::Deploy;
use crate::deploy::esp::find_esp_mount;
use crate::layout::boot_plugins::{BOOT_PLUGINS_DIR, MANIFEST_EXTENSION};
use crate::mutated::files::{FilesError, NewPrefixDigest, RequestedBootPlugin, ResolvedBootEntry};
use crate::orchestrator::stage::{NoRollback, RollbackGuard, Stage, StageResult};
use crate::orchestrator::{Context, ctx_get};
use crate::plugin::boot::resolve_boot_plugin;

pub struct CheckoutStage;

impl Stage<FilesError> for CheckoutStage {
    fn run(
        &self, context: &mut Context, _cancel: &CancelToken, progress: ProgressEventBuilder,
    ) -> Result<(ProgressEventBuilder, StageResult, Box<dyn RollbackGuard>), FilesError> {
        let new_prefix = ctx_get!(context, NewPrefixDigest);
        let deploy = ctx_get!(context, Deploy);
        let requested_boot_plugins = ctx_get!(context, RequestedBootPlugin);

        let repository = deploy.open_repository()?;
        let deploy_tree = deploy.open_tree(&new_prefix.0)?;
        let digest = object_id_from_hex(&new_prefix.0)?;

        let esp_mount = find_esp_mount()?;
        let entry_name = write_boot_entry(&repository, &deploy_tree, digest, &esp_mount, &new_prefix.0)?;

        let plugin = resolve_boot_plugin(
            BOOT_PLUGINS_DIR,
            MANIFEST_EXTENSION,
            requested_boot_plugins.0.as_deref(),
        )?;

        context.put(ResolvedBootEntry { plugin, entry_name });

        Ok((progress, StageResult::Advance, Box::new(NoRollback)))
    }
}
