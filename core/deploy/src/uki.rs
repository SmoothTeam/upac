// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::fs::{copy, create_dir_all, rename, write};
use std::io::ErrorKind as IoErrorKind;
use std::path::Path;
use std::process::{Command, Stdio};

use tempfile::TempDir;

use super::error::BootEntryError;
use super::layout::boot::UKIFY_BIN;

pub(crate) struct UkiParts<'parts> {
    pub kernel: &'parts [u8],
    pub initramfs: &'parts [u8],
    pub os_release: Option<&'parts [u8]>,
    pub cmdline: String,
}

impl UkiParts<'_> {
    pub fn build(&self, output: &Path) -> Result<(), BootEntryError> {
        let output_dir = output.parent().ok_or(BootEntryError::Unexpected)?;
        create_dir_all(output_dir)?;

        let scratch = TempDir::new()?;
        let kernel_path = scratch.path().join("vmlinuz");
        let initramfs_path = scratch.path().join("initramfs.img");
        write(&kernel_path, self.kernel)?;
        write(&initramfs_path, self.initramfs)?;

        let staged_output = scratch.path().join("image.efi");

        let mut command = Command::new(UKIFY_BIN);
        command
            .arg("build")
            .arg("--linux")
            .arg(&kernel_path)
            .arg("--initrd")
            .arg(&initramfs_path)
            .arg("--cmdline")
            .arg(&self.cmdline)
            .arg("--output")
            .arg(&staged_output);

        if let Some(os_release) = self.os_release {
            let os_release_path = scratch.path().join("os-release");
            write(&os_release_path, os_release)?;
            command
                .arg("--os-release")
                .arg(format!("@{}", os_release_path.display()));
        }

        let status = command
            .stdout(Stdio::null())
            .stderr(Stdio::inherit())
            .status()
            .map_err(|error| match error.kind() {
                IoErrorKind::NotFound => BootEntryError::ToolNotInstalled,
                _ => BootEntryError::from(error),
            })?;
        if !status.success() || !staged_output.is_file() {
            return Err(BootEntryError::ToolFailed);
        }

        let staged_on_esp = output_dir.join(format!(
            ".{}.staged",
            output.file_name().ok_or(BootEntryError::Unexpected)?.to_string_lossy()
        ));
        copy(&staged_output, &staged_on_esp)?;
        rename(&staged_on_esp, output)?;

        Ok(())
    }
}
