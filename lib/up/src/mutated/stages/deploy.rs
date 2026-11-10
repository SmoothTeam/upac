// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_types::CancelToken;
use upac_types::error::ErrorKind;

use upac_deploy::Sysroot;
use upac_deploy::deployment::{Deployment, PrefixDeploy};
use upac_deploy::working::CommittedPrefix;

use upac_macro::stage;

use upac_orchestrator::context::Context;
use upac_orchestrator::stage::Stage;

use super::{BootTarget, DeployedPrefix, NewConfig};

pub struct DeployStage;

#[stage]
impl Stage<ErrorKind> for DeployStage {
    fn run(
        &self, context: &mut Context, _cancel: &CancelToken, _progress: &dyn Fn(Option<&str>, u64, u64),
    ) -> Result<(), ErrorKind> {
        let committed = context.take::<CommittedPrefix>()?;
        let new_config = context.take::<NewConfig>()?;
        let sysroot = context.get::<Sysroot>()?;

        let prefix = PrefixDeploy::new(committed.digest, committed.transaction, new_config.0);
        sysroot.create_prefix(&prefix)?;

        let next_written = match sysroot.set_next_prefix(&prefix) {
            Ok(next_written) => next_written,
            Err(error) => {
                sysroot.remove_prefix(prefix.digest())?;
                return Err(error.into());
            }
        };

        let digest = prefix.digest().clone();
        context.put(BootTarget(digest.clone()));
        context.put(DeployedPrefix { digest, next_written });

        Ok(())
    }

    fn rollback(&self, context: &mut Context) -> Result<(), ErrorKind> {
        let Ok(deployed) = context.take::<DeployedPrefix>() else {
            return Ok(());
        };

        deployed.next_written.restore()?;
        context.get::<Sysroot>()?.remove_prefix(&deployed.digest)?;

        Ok(())
    }
}
