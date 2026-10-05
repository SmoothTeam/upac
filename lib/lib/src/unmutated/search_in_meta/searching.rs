// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_types::CancelToken;
use upac_types::error::ErrorKind;
use upac_types::package::PackageInfo;
use upac_types::response::unmutated::SearchInMetaResponse;

use upac_database::meta::MetaStore;

use upac_deploy::Sysroot;

use upac_macro::stage;

use upac_orchestrator::context::Context;
use upac_orchestrator::stage::Stage;

use super::super::running_prefix_database;

use crate::search::Search;

pub struct SearchingStage;

#[stage]
impl Stage<ErrorKind> for SearchingStage {
    fn run(
        &self, context: &mut Context, _cancel: &CancelToken, _progress: &dyn Fn(Option<&str>, u64, u64),
    ) -> Result<(), ErrorKind> {
        let package = context.get::<PackageInfo>()?;
        let search = context.get::<Search>()?;
        let database = running_prefix_database(context.get::<Sysroot>()?)?;

        let uuid = database
            .find_package_uuid(&package.name, &package.arch, package.arch_sub.as_deref())?
            .ok_or(ErrorKind::NotFound)?;
        let package_meta = database.get_package_meta(uuid)?.ok_or(ErrorKind::NotFound)?;

        let metas = if search.is_match(&package_meta.name) || search.is_match(&package_meta.description) {
            vec![package_meta]
        } else {
            Vec::new()
        };

        context.put(SearchInMetaResponse { metas });

        Ok(())
    }
}
