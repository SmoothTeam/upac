// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::fs::read_dir;
use std::os::unix::fs::symlink;

use upac_types::CancelToken;
use upac_types::error::ErrorKind;

use upac_composefs::tree::Tree;

use upac_deploy::layout::prefix::SYSTEM_PREFIX_DIR;

use upac_macro::stage;

use upac_orchestrator::context::Context;
use upac_orchestrator::stage::Stage;

use super::{ExportDir, KernelVersion};

use crate::layout::genesis::MODULES_DIR;

const MERGED_USR_LINKS: &[(&str, &str)] = &[
    ("bin", "usr/bin"),
    ("sbin", "usr/bin"),
    ("lib", "usr/lib"),
    ("lib64", "usr/lib"),
];

pub struct ExportStage;

#[stage]
impl Stage<ErrorKind> for ExportStage {
    fn run(
        &self, context: &mut Context, cancel: &CancelToken, _progress: &dyn Fn(Option<&str>, u64, u64),
    ) -> Result<(), ErrorKind> {
        let export_dir = &context.get::<ExportDir>()?.0;
        let prefix_dir = export_dir.join(SYSTEM_PREFIX_DIR);

        context.get::<Tree>()?.export_dir("", &prefix_dir, cancel)?;
        for (link_name, link_target) in MERGED_USR_LINKS {
            symlink(link_target, export_dir.join(link_name))?;
        }

        let mut versions = Vec::new();
        for entry in read_dir(prefix_dir.join(MODULES_DIR))? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                versions.push(entry.file_name().to_string_lossy().into_owned());
            }
        }

        let version = match versions.as_slice() {
            [] => return Err(ErrorKind::NotFound),
            [version] => version.clone(),
            _ => return Err(ErrorKind::InvalidEntry),
        };

        context.put(KernelVersion(version));

        Ok(())
    }
}
