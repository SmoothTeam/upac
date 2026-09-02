// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_abi::hook::{CancelToken, ProgressEventBuilder};

use crate::boot::write_boot_entry;
use crate::composefs::repository::object_id_from_hex;
use crate::deploy::Deploy;
use crate::deploy::esp::find_esp_mount;
use crate::errors::CommonError;
use crate::layout::boot_plugins::{BOOT_PLUGINS_DIR, MANIFEST_EXTENSION};
use crate::mutated::uninstaller::{NewPrefixDigest, RequestedBootPlugin, ResolvedBootEntry, UninstallError};
use crate::orchestrator::Context;
use crate::orchestrator::stage::{NoRollback, RollbackGuard, Stage, StageResult};
use crate::plugin::boot::resolve_boot_plugin;

pub struct CheckoutStage;

impl Stage<UninstallError> for CheckoutStage {
    fn run(
        &self, context: &mut Context, _cancel: &CancelToken, progress: ProgressEventBuilder,
    ) -> Result<(ProgressEventBuilder, StageResult, Box<dyn RollbackGuard>), UninstallError> {
        let new_prefix = context.get::<NewPrefixDigest>().ok_or(CommonError::MissingResult)?;
        let deploy = context.get::<Deploy>().ok_or(CommonError::MissingResult)?;
        let requested = context.get::<RequestedBootPlugin>().ok_or(CommonError::MissingResult)?;

        let repository = deploy.open_repository()?;
        let tree = deploy.open_tree(&new_prefix.0)?;
        let digest = object_id_from_hex(&new_prefix.0)?;

        let esp_mount = find_esp_mount()?;
        let entry_name = write_boot_entry(&repository, &tree, digest, &esp_mount, &new_prefix.0)?;

        let plugin = resolve_boot_plugin(BOOT_PLUGINS_DIR, MANIFEST_EXTENSION, requested.0.as_deref())?;

        context.put(ResolvedBootEntry { plugin, entry_name });

        Ok((progress, StageResult::Advance, Box::new(NoRollback)))
    }
}
