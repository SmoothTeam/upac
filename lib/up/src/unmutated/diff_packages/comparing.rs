// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::collections::HashMap;

use upac_types::CancelToken;
use upac_types::diff::PackageDiffKind;
use upac_types::error::ErrorKind;
use upac_types::response::entry::DiffPackageEntry;
use upac_types::response::unmutated::DiffPackagesResponse;

use upac_macro::stage;

use upac_orchestrator::context::Context;
use upac_orchestrator::stage::Stage;

use super::DiffPackagesSnapshot;

pub struct ComparingStage;

#[stage]
impl Stage<ErrorKind> for ComparingStage {
    fn run(
        &self, context: &mut Context, _cancel: &CancelToken, _progress: &dyn Fn(Option<&str>, u64, u64),
    ) -> Result<(), ErrorKind> {
        let snapshot = context.take::<DiffPackagesSnapshot>()?;

        let from: HashMap<_, _> = snapshot
            .from
            .into_iter()
            .map(|meta| ((meta.name.clone(), meta.arch.clone(), meta.arch_sub.clone()), meta))
            .collect();
        let mut to: HashMap<_, _> = snapshot
            .to
            .into_iter()
            .map(|meta| ((meta.name.clone(), meta.arch.clone(), meta.arch_sub.clone()), meta))
            .collect();

        let mut entries = Vec::new();

        for (identity, from_meta) in from {
            match to.remove(&identity) {
                Some(to_meta) if to_meta.sha256 != from_meta.sha256 => entries.push(DiffPackageEntry {
                    name: to_meta.name,
                    kind: PackageDiffKind::Modified,
                    version: to_meta.version,
                    files: Vec::new(),
                }),
                Some(_) => {}
                None => entries.push(DiffPackageEntry {
                    name: from_meta.name,
                    kind: PackageDiffKind::Removed,
                    version: from_meta.version,
                    files: Vec::new(),
                }),
            }
        }

        for (_identity, to_meta) in to {
            entries.push(DiffPackageEntry {
                name: to_meta.name,
                kind: PackageDiffKind::Added,
                version: to_meta.version,
                files: Vec::new(),
            });
        }

        context.put(DiffPackagesResponse { diff_packages: entries });

        Ok(())
    }
}
