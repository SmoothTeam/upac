// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_types::CancelToken;
use upac_types::error::ErrorKind;
use upac_types::package::PackageInfo;
use upac_types::response::entry::SearchFileEntry;
use upac_types::response::unmutated::SearchInPackageFilesResponse;

use upac_database::files::FileStore;
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

        let files = database
            .list_package_files(uuid)?
            .into_iter()
            .filter(|entry| search.is_match(&entry.path))
            .map(|entry| SearchFileEntry {
                path: entry.path,
                package_name: package.name.clone(),
                is_user: entry.is_user,
            })
            .collect();

        context.put(SearchInPackageFilesResponse { files });

        Ok(())
    }
}
