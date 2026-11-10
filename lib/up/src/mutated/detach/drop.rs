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

use super::super::stages::{DetachItem, FileOwner, RequestedScope};

pub struct DropStage;

#[stage]
impl Stage<ErrorKind> for DropStage {
    fn run(
        &self, context: &mut Context, _cancel: &CancelToken, _progress: &dyn Fn(Option<&str>, u64, u64),
    ) -> Result<(), ErrorKind> {
        let item = context.take::<DetachItem>()?;
        let mut working = context.take::<WorkingPrefix>()?;
        let owner = context.get::<FileOwner>()?.0;
        let scope = context.get::<RequestedScope>()?.0;

        working.detach_file(owner, scope, &item.0)?;

        context.put(working);

        Ok(())
    }
}
