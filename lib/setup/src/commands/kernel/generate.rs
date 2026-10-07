// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::io::ErrorKind as IoErrorKind;
use std::path::Path;
use std::process::{Command, Stdio};

use upac_types::CancelToken;
use upac_types::booter::BootResourceKind;
use upac_types::error::ErrorKind;
use upac_types::request::bootstrap::InitramfsGenerator;

use upac_boot_loader::BootPlugins;

use upac_deploy::layout::prefix::SYSTEM_PREFIX_DIR;

use upac_macro::stage;

use upac_orchestrator::context::Context;
use upac_orchestrator::stage::Stage;

use super::{ExportDir, KernelImage, KernelVersion, RequestedBootPlugin, RequestedInitramfsGenerator};

use crate::layout::genesis::{INITRAMFS_FILENAME, MODULES_DIR, UKI_FILENAME};
use crate::layout::initramfs::{DEPMOD_BIN, DRACUT_BIN, MKINITCPIO_BIN};

pub struct GenerateStage;

#[stage]
impl Stage<ErrorKind> for GenerateStage {
    fn run(
        &self, context: &mut Context, _cancel: &CancelToken, _progress: &dyn Fn(Option<&str>, u64, u64),
    ) -> Result<(), ErrorKind> {
        let export_dir = &context.get::<ExportDir>()?.0;
        let version = &context.get::<KernelVersion>()?.0;
        let generator = context.get::<RequestedInitramfsGenerator>()?.0;

        let is_uki = BootPlugins::new()?
            .load(&context.get::<RequestedBootPlugin>()?.0)?
            .boot_resource_kind()?
            == BootResourceKind::Uki;
        let image_name = if is_uki { UKI_FILENAME } else { INITRAMFS_FILENAME };
        let image_path = export_dir
            .join(SYSTEM_PREFIX_DIR)
            .join(MODULES_DIR)
            .join(version)
            .join(image_name);

        Self::run_tool(Command::new(DEPMOD_BIN).arg("-b").arg(export_dir).arg(version))?;

        match generator {
            InitramfsGenerator::Dracut => Self::run_dracut(export_dir, version, is_uki, &image_path)?,
            InitramfsGenerator::Mkinitcpio => Self::run_mkinitcpio(export_dir, version, is_uki, &image_path)?,
        }

        context.put(KernelImage(image_path));

        Ok(())
    }
}

impl GenerateStage {
    fn run_dracut(export_dir: &Path, version: &str, is_uki: bool, image_path: &Path) -> Result<(), ErrorKind> {
        let mut command = Command::new(DRACUT_BIN);
        command.arg("--sysroot").arg(export_dir).args([
            "--no-hostonly",
            "--no-hostonly-cmdline",
            "--force",
            "--kver",
            version,
        ]);
        if is_uki {
            command.arg("--uefi");
        }
        command.arg(image_path);

        Self::run_tool(&mut command)
    }

    fn run_mkinitcpio(export_dir: &Path, version: &str, is_uki: bool, image_path: &Path) -> Result<(), ErrorKind> {
        let mut command = Command::new(MKINITCPIO_BIN);
        command
            .args(["-k", version, "-r"])
            .arg(export_dir)
            .args(["-S", "autodetect"])
            .arg(if is_uki { "-U" } else { "-g" })
            .arg(image_path);

        Self::run_tool(&mut command)
    }

    fn run_tool(command: &mut Command) -> Result<(), ErrorKind> {
        let status = command
            .stdout(Stdio::null())
            .stderr(Stdio::inherit())
            .status()
            .map_err(|error| match error.kind() {
                IoErrorKind::NotFound => ErrorKind::ToolNotInstalled,
                _ => ErrorKind::from(error),
            })?;

        if !status.success() {
            return Err(ErrorKind::ToolFailed);
        }

        Ok(())
    }
}
