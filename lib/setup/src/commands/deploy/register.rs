// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_types::CancelToken;
use upac_types::error::ErrorKind;

use upac_database::transaction::TransactionStore;

use upac_deploy::Sysroot;
use upac_deploy::deployment::config::ConfigDeploy;
use upac_deploy::deployment::{Deployment, PrefixDeploy};

use upac_macro::stage;

use upac_orchestrator::context::Context;
use upac_orchestrator::stage::Stage;

use super::{DeployedPrefix, RequestedDeploy};

use crate::layout::genesis::SUBJECT;

pub struct RegisterStage;

#[stage]
impl Stage<ErrorKind> for RegisterStage {
    fn run(
        &self, context: &mut Context, _cancel: &CancelToken, _progress: &dyn Fn(Option<&str>, u64, u64),
    ) -> Result<(), ErrorKind> {
        let sysroot = context.get::<Sysroot>()?;
        let requested = context.get::<RequestedDeploy>()?;

        let transaction = sysroot
            .prefix_database(&requested.prefix_digest)?
            .get_transaction()?
            .ok_or(ErrorKind::NotFound)?;
        let first_config = ConfigDeploy::new(requested.config_digest.clone(), SUBJECT.to_owned(), None);

        let mut prefix = PrefixDeploy::new(requested.prefix_digest.clone(), transaction, first_config);
        prefix.set_pinned(requested.pinned);
        sysroot.create_prefix(&prefix)?;

        let next_written = match sysroot.set_next_prefix(&prefix) {
            Ok(next_written) => next_written,
            Err(error) => {
                sysroot.remove_prefix(prefix.digest())?;
                return Err(error.into());
            }
        };

        context.put(DeployedPrefix {
            digest: prefix.digest().clone(),
            next_written,
        });

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
