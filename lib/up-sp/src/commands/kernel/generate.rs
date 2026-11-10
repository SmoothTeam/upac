// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::io::ErrorKind as IoErrorKind;
use std::path::Path;
use std::process::{Command, Stdio};

use upac_types::CancelToken;
use upac_types::error::ErrorKind;
use upac_types::request::bootstrap::InitramfsGenerator;

use upac_deploy::layout::prefix::SYSTEM_PREFIX_DIR;

use upac_macro::stage;

use upac_orchestrator::context::Context;
use upac_orchestrator::stage::Stage;

use super::{ExportDir, KernelImage, KernelVersion, RequestedInitramfsGenerator};

use crate::layout::bootstrap::{INITRAMFS_FILENAME, MODULES_DIR};
use crate::layout::initramfs::{DEPMOD_BIN, DRACUT_BIN, MKINITCPIO_BIN};

pub struct GenerateStage;

#[stage]
impl Stage<ErrorKind> for GenerateStage {
    fn run(
        &self, context: &mut Context, _cancel: &CancelToken, progress: &dyn Fn(Option<&str>, u64, u64),
    ) -> Result<(), ErrorKind> {
        let export_dir = &context.get::<ExportDir>()?.0;
        let version = &context.get::<KernelVersion>()?.0;
        let generator = context.get::<RequestedInitramfsGenerator>()?.0;

        let image_path = export_dir
            .join(SYSTEM_PREFIX_DIR)
            .join(MODULES_DIR)
            .join(version)
            .join(INITRAMFS_FILENAME);

        Self::run_tool(Command::new(DEPMOD_BIN).arg("-b").arg(export_dir).arg(version))?;

        progress(Some(&image_path.to_string_lossy()), 0, 0);
        match generator {
            InitramfsGenerator::Dracut => Self::run_dracut(export_dir, version, &image_path)?,
            InitramfsGenerator::Mkinitcpio => Self::run_mkinitcpio(export_dir, version, &image_path)?,
        }
        if !image_path.is_file() {
            return Err(ErrorKind::ToolFailed);
        }

        context.put(KernelImage(image_path));

        Ok(())
    }
}

impl GenerateStage {
    fn run_dracut(export_dir: &Path, version: &str, image_path: &Path) -> Result<(), ErrorKind> {
        let mut command = Command::new(DRACUT_BIN);
        command.arg("--sysroot").arg(export_dir).args([
            "--no-hostonly",
            "--no-hostonly-cmdline",
            "--force",
            "--kver",
            version,
        ]);
        command.arg(image_path);

        Self::run_tool(&mut command)
    }

    fn run_mkinitcpio(export_dir: &Path, version: &str, image_path: &Path) -> Result<(), ErrorKind> {
        let mut command = Command::new(MKINITCPIO_BIN);
        command
            .args(["-k", version, "-r"])
            .arg(export_dir)
            .args(["-S", "autodetect"])
            .arg("-g")
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
