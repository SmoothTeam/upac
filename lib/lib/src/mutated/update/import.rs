// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::cmp::Ordering;
use std::fs::remove_dir_all;
use std::path::Path;

use upac_types::CancelToken;
use upac_types::error::ErrorKind;

use upac_database::meta::MetaStore;

use upac_deploy::working::WorkingPrefix;

use upac_macro::stage;

use upac_orchestrator::context::Context;
use upac_orchestrator::stage::Stage;

use super::super::stages::UnpackedPackage;
use super::AllowDowngrade;

pub struct ImportStage;

#[stage]
impl Stage<ErrorKind> for ImportStage {
    fn run(
        &self, context: &mut Context, cancel: &CancelToken, progress: &dyn Fn(Option<&str>, u64, u64),
    ) -> Result<(), ErrorKind> {
        let package = context.take::<UnpackedPackage>()?;
        let mut working = context.take::<WorkingPrefix>()?;
        let allow_downgrade = context.get::<AllowDowngrade>()?.0;

        let meta = &package.temp.meta;
        progress(Some(&meta.name), 0, 0);
        let uuid = working
            .database()
            .find_package_uuid(&meta.name, &meta.arch, meta.arch_sub.as_deref())?
            .ok_or(ErrorKind::NotFound)?;
        let installed_meta = working.database().get_package_meta(uuid)?.ok_or(ErrorKind::NotFound)?;

        match meta.version.cmp(&installed_meta.version) {
            Ordering::Equal => return Err(ErrorKind::AlreadyExists),
            Ordering::Less if !allow_downgrade => return Err(ErrorKind::InvalidEntry),
            Ordering::Less | Ordering::Greater => {}
        }

        let unpacked_root = Path::new(&package.temp.temp_package_path);
        working.replace_package(uuid, meta, &package.triggers, unpacked_root, cancel)?;
        remove_dir_all(unpacked_root)?;

        context.put(working);

        Ok(())
    }
}
