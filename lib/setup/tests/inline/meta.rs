// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::fs::{create_dir_all, write};

use tempfile::TempDir;

use upac::orchestrator::Context;
use upac::orchestrator::stage::{Stage, StageResult};

use upac_abi::hook::{CancelToken, ProgressEventBuilder};

use upac_types::PackageMeta;

use crate::types::{GenesisInput, ResolvedSourceDir};

use super::ReadMetaStage;

fn genesis_input(empty_config: bool) -> GenesisInput {
    GenesisInput {
        source: String::new(),
        meta_filename: None,
        empty_config,
        pinned: false,
        boot_plugin: None,
    }
}

#[test]
fn run_reads_meta_and_fills_in_sha256_and_installed_size() {
    let scratch = TempDir::new().unwrap();
    write(
        scratch.path().join("meta.toml"),
        "name = \"test-pkg\"\narch = \"x86_64\"\n",
    )
    .unwrap();
    create_dir_all(scratch.path().join("usr")).unwrap();
    write(scratch.path().join("usr/a.txt"), b"hello").unwrap();

    let mut context = Context::new();
    context.put(genesis_input(false));
    context.put(ResolvedSourceDir(scratch.path().to_path_buf()));

    let cancel = CancelToken::new();
    let progress = ProgressEventBuilder::new(0);

    let (_, result, _guard) = ReadMetaStage.run(&mut context, &cancel, progress).unwrap();

    assert!(matches!(result, StageResult::Advance));

    let meta = context.get::<PackageMeta>().unwrap();
    assert_eq!(meta.name, "test-pkg");
    assert_eq!(meta.arch, "x86_64");
    assert_eq!(meta.installed_size, 5);
    assert_ne!(meta.sha256, [0u8; 32]);
}

#[test]
fn run_fails_when_meta_toml_missing() {
    let scratch = TempDir::new().unwrap();

    let mut context = Context::new();
    context.put(genesis_input(false));
    context.put(ResolvedSourceDir(scratch.path().to_path_buf()));

    let cancel = CancelToken::new();
    let progress = ProgressEventBuilder::new(0);

    let result = ReadMetaStage.run(&mut context, &cancel, progress);

    assert!(result.is_err());
}
