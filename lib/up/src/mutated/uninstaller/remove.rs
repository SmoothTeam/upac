// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_types::CancelToken;
use upac_types::error::ErrorKind;

use upac_deploy::working::WorkingPrefix;

use upac_macro::stage;

use upac_orchestrator::context::Context;
use upac_orchestrator::stage::Stage;

use super::{Purge, RemovalTarget};

pub struct RemoveStage;

#[stage]
impl Stage<ErrorKind> for RemoveStage {
    fn run(
        &self, context: &mut Context, _cancel: &CancelToken, _progress: &dyn Fn(Option<&str>, u64, u64),
    ) -> Result<(), ErrorKind> {
        let target = context.take::<RemovalTarget>()?;
        let mut working = context.take::<WorkingPrefix>()?;
        let purge = context.get::<Purge>()?.0;

        working.remove_package(target.0, purge)?;

        context.put(working);

        Ok(())
    }
}
