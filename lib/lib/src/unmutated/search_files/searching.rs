// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_types::CancelToken;
use upac_types::error::ErrorKind;
use upac_types::response::entry::SearchFileEntry;
use upac_types::response::unmutated::SearchFilesResponse;

use upac_database::files::FileStore;
use upac_database::meta::MetaStore;

use upac_deploy::Sysroot;

use upac_macro::stage;

use upac_orchestrator::context::Context;
use upac_orchestrator::stage::Stage;

use super::super::running_prefix_database;

use crate::Search;

pub struct SearchingStage;

#[stage]
impl Stage<ErrorKind> for SearchingStage {
    fn run(
        &self, context: &mut Context, _cancel: &CancelToken, _progress: &dyn Fn(Option<&str>, u64, u64),
    ) -> Result<(), ErrorKind> {
        let search = context.get::<Search>()?;
        let database = running_prefix_database(context.get::<Sysroot>()?)?;

        let mut files = Vec::new();
        for (uuid, file_entry) in database.list_files()? {
            if !search.is_match(&file_entry.path) {
                continue;
            }

            let Some(package_meta) = database.get_package_meta(uuid)? else {
                continue;
            };

            files.push(SearchFileEntry {
                path: file_entry.path,
                package_name: package_meta.name,
                is_user: file_entry.is_user,
            });
        }

        context.put(SearchFilesResponse { files });

        Ok(())
    }
}
