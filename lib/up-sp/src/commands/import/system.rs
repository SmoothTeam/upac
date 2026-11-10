// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_types::CancelToken;
use upac_types::error::ErrorKind;

use upac_deploy::layout::deployment::SYSROOT_DIR;
use upac_deploy::working::WorkingPrefix;

use upac_macro::stage;

use upac_orchestrator::context::Context;
use upac_orchestrator::stage::Stage;

use super::SourceDir;

use crate::layout::bootstrap::{COMPOSEFS_SETUP_ROOT_UNIT_PATH, SYSTEM_DIR};

pub struct SystemStage;

#[stage]
impl Stage<ErrorKind> for SystemStage {
    fn run(
        &self, context: &mut Context, cancel: &CancelToken, progress: &dyn Fn(Option<&str>, u64, u64),
    ) -> Result<(), ErrorKind> {
        let system_dir = context.get::<SourceDir>()?.0.join(SYSTEM_DIR);

        let unit_path = system_dir.join(COMPOSEFS_SETUP_ROOT_UNIT_PATH);
        progress(Some(&unit_path.to_string_lossy()), 0, 0);
        if !unit_path.is_file() {
            return Err(ErrorKind::NotFound);
        }

        let sysroot_dir = system_dir.join(SYSROOT_DIR);
        progress(Some(&sysroot_dir.to_string_lossy()), 0, 0);
        if !sysroot_dir.is_dir() {
            return Err(ErrorKind::NotFound);
        }

        let mut working = context.take::<WorkingPrefix>()?;
        let added = working.add_unowned_dir(&system_dir, cancel, &mut |path| {
            progress(Some(&path.to_string_lossy()), 0, 0)
        });
        context.put(working);
        added?;

        Ok(())
    }
}
