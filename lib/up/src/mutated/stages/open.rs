// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_types::CancelToken;
use upac_types::error::ErrorKind;

use upac_deploy::Sysroot;
use upac_deploy::deployment::Deployment;
use upac_deploy::error::{PrefixMetaError, PrefixReadError};
use upac_deploy::working::WorkingPrefix;

use upac_macro::stage;

use upac_orchestrator::context::Context;
use upac_orchestrator::stage::Stage;

use super::RunningPrefix;

pub struct OpenStage;

#[stage]
impl Stage<ErrorKind> for OpenStage {
    fn run(
        &self, context: &mut Context, _cancel: &CancelToken, _progress: &dyn Fn(Option<&str>, u64, u64),
    ) -> Result<(), ErrorKind> {
        let sysroot = context.get::<Sysroot>()?;

        let running_prefix = sysroot.running_prefix()?;
        let base_prefix = match sysroot.next_prefix() {
            Ok(next_prefix) if next_prefix.digest() != running_prefix.digest() => next_prefix,
            Ok(_) | Err(PrefixReadError::Meta(PrefixMetaError::NotFound)) => running_prefix.clone(),
            Err(error) => return Err(error.into()),
        };

        let working = sysroot.working_prefix(&base_prefix)?;
        context.put::<WorkingPrefix>(working);
        context.put(RunningPrefix(running_prefix));

        Ok(())
    }
}
