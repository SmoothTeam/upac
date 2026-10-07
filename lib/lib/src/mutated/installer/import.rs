// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::fs::remove_dir_all;
use std::path::Path;

use upac_types::CancelToken;
use upac_types::error::ErrorKind;

use upac_deploy::working::WorkingPrefix;

use upac_macro::stage;

use upac_orchestrator::context::Context;
use upac_orchestrator::stage::Stage;

use super::super::stages::UnpackedPackage;

pub struct ImportStage;

#[stage]
impl Stage<ErrorKind> for ImportStage {
    fn run(
        &self, context: &mut Context, cancel: &CancelToken, _progress: &dyn Fn(Option<&str>, u64, u64),
    ) -> Result<(), ErrorKind> {
        let package = context.take::<UnpackedPackage>()?;
        let mut working = context.take::<WorkingPrefix>()?;

        let unpacked_root = Path::new(&package.temp.temp_package_path);
        working.add_package(&package.temp.meta, &package.triggers, unpacked_root, cancel)?;
        remove_dir_all(unpacked_root)?;

        context.put(working);

        Ok(())
    }
}
