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

use crate::layout::genesis::{COMPOSEFS_SETUP_ROOT_UNIT_PATH, SYSTEM_DIR};

pub struct SystemStage;

#[stage]
impl Stage<ErrorKind> for SystemStage {
    fn run(
        &self, context: &mut Context, cancel: &CancelToken, _progress: &dyn Fn(Option<&str>, u64, u64),
    ) -> Result<(), ErrorKind> {
        let system_dir = context.get::<SourceDir>()?.0.join(SYSTEM_DIR);

        if !system_dir.join(COMPOSEFS_SETUP_ROOT_UNIT_PATH).is_file() || !system_dir.join(SYSROOT_DIR).is_dir() {
            return Err(ErrorKind::NotFound);
        }

        let mut working = context.take::<WorkingPrefix>()?;
        working.add_unowned_dir(&system_dir, cancel)?;
        context.put(working);

        Ok(())
    }
}
