// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::fs::{metadata, read_dir};

use tempfile::TempDir;

use upac_types::CancelToken;
use upac_types::error::ErrorKind;

use upac_macro::stage;

use upac_orchestrator::context::Context;
use upac_orchestrator::stage::Stage;

use super::{PackageSource, RequestedSource, ScratchDir, SourceDir};

use crate::archive::SourceArchive;

pub struct PrepareStage;

#[stage]
impl Stage<ErrorKind> for PrepareStage {
    fn run(
        &self, context: &mut Context, _cancel: &CancelToken, _progress: &dyn Fn(Option<&str>, u64, u64),
    ) -> Result<(), ErrorKind> {
        let source = context.take::<RequestedSource>()?.0;

        let source_dir = if metadata(&source)?.is_dir() {
            source
        } else {
            let extracted_dir = TempDir::new_in(&context.get::<ScratchDir>()?.0)?.keep();
            SourceArchive::sniff(&source)?.extract(&extracted_dir)?;
            extracted_dir
        };

        let mut package_paths = Vec::new();
        for entry in read_dir(&source_dir)? {
            let entry = entry?;
            if entry.metadata()?.is_file() {
                package_paths.push(entry.path());
            }
        }
        package_paths.sort();

        let sources: Vec<PackageSource> = package_paths
            .into_iter()
            .enumerate()
            .map(|(index, path)| PackageSource { path, index })
            .collect();

        context.put::<Vec<PackageSource>>(sources);
        context.put(SourceDir(source_dir));

        Ok(())
    }
}
