// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_types::CancelToken;
use upac_types::diff::DiffFileSource;
use upac_types::error::ErrorKind;

use upac_database::meta::MetaStore;

use upac_deploy::Sysroot;
use upac_deploy::deployment::Deployment;

use upac_macro::stage;

use upac_orchestrator::context::Context;
use upac_orchestrator::stage::Stage;

use super::super::{RequestedConfigDigestRange, RequestedPrefixDigestRange, requested_prefix, requested_prefix_config};
use super::DiffSnapshot;

pub struct PreparingStage;

#[stage]
impl Stage<ErrorKind> for PreparingStage {
    fn run(
        &self, context: &mut Context, _cancel: &CancelToken, _progress: &dyn Fn(Option<&str>, u64, u64),
    ) -> Result<(), ErrorKind> {
        let sysroot = context.get::<Sysroot>()?;
        let requested_prefixes = context.get::<RequestedPrefixDigestRange>()?;
        let requested_configs = context.get::<RequestedConfigDigestRange>()?;

        let from_prefix = requested_prefix(sysroot, requested_prefixes.from.as_ref())?;
        let to_prefix = requested_prefix(sysroot, requested_prefixes.to.as_ref())?;

        let from_config = requested_prefix_config(&from_prefix, requested_configs.from.as_ref())?;
        let to_config = requested_prefix_config(&to_prefix, requested_configs.to.as_ref())?;

        let repo = sysroot.repo();
        let prefix_changes = repo
            .open_tree(from_prefix.digest())?
            .diff(&repo.open_tree(to_prefix.digest())?)
            .into_iter()
            .map(|(path, kind)| (path, kind, DiffFileSource::Prefix));
        let config_changes = repo
            .open_tree(from_config.digest())?
            .diff(&repo.open_tree(to_config.digest())?)
            .into_iter()
            .map(|(path, kind)| (path, kind, DiffFileSource::Config));
        let changed_files = prefix_changes.chain(config_changes).collect();

        let from_database = sysroot.prefix_database(from_prefix.digest())?;
        let to_database = sysroot.prefix_database(to_prefix.digest())?;
        let from_packages = from_database.list_packages_metas()?;
        let to_packages = to_database.list_packages_metas()?;

        context.put(DiffSnapshot {
            from_packages,
            to_packages,
            changed_files,
            from_database,
            to_database,
        });

        Ok(())
    }
}
