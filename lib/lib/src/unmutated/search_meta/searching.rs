// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_types::CancelToken;
use upac_types::error::ErrorKind;
use upac_types::response::unmutated::SearchMetaResponse;

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
        let search = context.get::<Search>()?;

        let metas = running_prefix_database(context.get::<Sysroot>()?)?
            .list_packages_metas()?
            .into_iter()
            .filter(|meta| search.is_match(&meta.name) || search.is_match(&meta.description))
            .collect();

        context.put(SearchMetaResponse { metas });

        Ok(())
    }
}
