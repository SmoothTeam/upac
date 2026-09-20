// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::fs::File;
use std::path::{Path, PathBuf};
use std::process::Command;

use composefs::generic_tree::Stat;
use composefs::repository::ImportContext;
use composefs::tree::FileSystem;

use tempfile::TempDir;

use upac::composefs::file::FileHandle;
use upac::composefs::repository::ObjectID;
use upac::orchestrator::context::{Context, ctx_get, ctx_take};
use upac::orchestrator::stage::{NoRollback, RollbackGuard, Stage, StageResult};
use upac::plugin::boot::BootPlugins;

use upac_abi::hook::CancelToken;
use upac_abi::{BootResourceKind, InitramfsGenerator};

use upac_types::hook::ProgressEventBuilder;

use super::{PrefixTree, RequestedBootPlugin, RequestedInitramfsGenerator};

use crate::error::SetupError;
use crate::layout::genesis::{INITRAMFS_FILENAME, UKI_FILENAME};
use crate::target::TargetSysroot;

pub struct KernelStage;

impl Stage<SetupError> for KernelStage {
    fn run(
        &self, context: &mut Context, cancel: &CancelToken, progress: ProgressEventBuilder,
    ) -> Result<(ProgressEventBuilder, StageResult, Box<dyn RollbackGuard>), SetupError> {
        let mut prefix_tree = ctx_take!(context, PrefixTree);
        let mut import_ctx = ctx_take!(context, ImportContext);

        let target = ctx_get!(context, TargetSysroot);
        let requested_boot_plugin = ctx_get!(context, RequestedBootPlugin);
        let requested_generator = ctx_get!(context, RequestedInitramfsGenerator);

        let repository = target.repository();

        let kver = detect_kernel_version(&prefix_tree)?;
        let plugin = BootPlugins::new()?.load(requested_boot_plugin)?;
        let is_uki = plugin.boot_resource_kind() == BootResourceKind::Uki;

        let scratch = TempDir::new()?;
        FileHandle::new(PathBuf::new()).export_directory(repository, &prefix_tree, scratch.path(), cancel)?;

        let output_name = if is_uki { UKI_FILENAME } else { INITRAMFS_FILENAME };
        let output_path = scratch.path().join("lib/modules").join(&kver).join(output_name);

        match **requested_generator {
            InitramfsGenerator::Dracut => run_dracut(scratch.path(), &kver, is_uki, &output_path)?,
            InitramfsGenerator::Mkinitcpio => run_mkinitcpio(scratch.path(), &kver, is_uki, &output_path)?,
        }

        FileHandle::new(format!("lib/modules/{kver}/{output_name}")).insert_file(
            repository,
            &mut prefix_tree,
            &File::open(&output_path)?,
            Stat::uninitialized(),
            &mut import_ctx,
        )?;

        context.put(import_ctx);
        context.put(prefix_tree);

        Ok((progress, StageResult::Advance, Box::new(NoRollback)))
    }
}

fn detect_kernel_version(prefix_tree: &FileSystem<ObjectID>) -> Result<String, SetupError> {
    let mut versions: Vec<String> = FileHandle::new("lib/modules")
        .list_in_tree(prefix_tree)?
        .map(|(name, _)| name.to_string_lossy().into_owned())
        .collect();

    match versions.len() {
        0 => Err(SetupError::NoKernelFound),
        1 => Ok(versions.remove(0)),
        _ => Err(SetupError::AmbiguousKernelVersion),
    }
}

fn run_dracut(scratch: &Path, kver: &str, is_uki: bool, output_path: &Path) -> Result<(), SetupError> {
    let mut command = Command::new("dracut");
    command
        .arg("--sysroot")
        .arg(scratch)
        .args(["--no-hostonly", "--no-hostonly-cmdline", "--force", "--kver", kver]);

    if is_uki {
        command.arg("--uefi");
    }

    command.arg(output_path);

    let status = command.status()?;
    if !status.success() {
        return Err(SetupError::InitramfsGeneratorFailed);
    }

    Ok(())
}

fn run_mkinitcpio(scratch: &Path, kver: &str, is_uki: bool, output_path: &Path) -> Result<(), SetupError> {
    let mut command = Command::new("mkinitcpio");
    command
        .args(["-k", kver, "-r"])
        .arg(scratch.join("lib/modules"))
        .args(["-S", "autodetect"])
        .arg(if is_uki { "-U" } else { "-g" })
        .arg(output_path);

    let status = command.status()?;
    if !status.success() {
        return Err(SetupError::InitramfsGeneratorFailed);
    }

    Ok(())
}
