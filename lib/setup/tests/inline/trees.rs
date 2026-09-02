// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::fs::{create_dir_all, write};
use std::path::PathBuf;

use tempfile::TempDir;

use upac::orchestrator::Context;
use upac::orchestrator::stage::{Stage, StageResult};

use upac_abi::hook::{CancelToken, ProgressEventBuilder};

use crate::target::TargetSysroot;
use crate::types::{ConfigTree, GenesisInput, ImportedConfigPaths, ImportedPrefixPaths, PrefixTree, ResolvedSourceDir};

use super::ImportTreesStage;

fn genesis_input(empty_config: bool) -> GenesisInput {
    GenesisInput {
        source: String::new(),
        meta_filename: None,
        empty_config,
        pinned: false,
        boot_plugin: None,
    }
}

fn context_with(source_dir: PathBuf, empty_config: bool) -> (Context, TempDir) {
    let target_scratch = TempDir::new().unwrap();
    let target = TargetSysroot::for_testing(target_scratch.path().to_path_buf()).unwrap();

    let mut context = Context::new();
    context.put(target);
    context.put(genesis_input(empty_config));
    context.put(ResolvedSourceDir(source_dir));

    (context, target_scratch)
}

#[test]
fn run_imports_usr_and_etc_and_records_their_paths() {
    let source = TempDir::new().unwrap();
    create_dir_all(source.path().join("usr/bin")).unwrap();
    write(source.path().join("usr/bin/tool"), b"binary").unwrap();
    create_dir_all(source.path().join("etc")).unwrap();
    write(source.path().join("etc/conf"), b"config").unwrap();

    let (mut context, _target_scratch) = context_with(source.path().to_path_buf(), false);
    let cancel = CancelToken::new();
    let progress = ProgressEventBuilder::new(0);

    let (_, result, _guard) = ImportTreesStage.run(&mut context, &cancel, progress).unwrap();

    assert!(matches!(result, StageResult::Advance));

    let prefix_paths = context.get::<ImportedPrefixPaths>().unwrap();
    assert_eq!(prefix_paths.0, vec![PathBuf::from("bin/tool")]);

    let config_paths = context.get::<ImportedConfigPaths>().unwrap();
    assert_eq!(config_paths.0, vec![PathBuf::from("conf")]);

    assert!(context.get::<PrefixTree>().is_some());
    assert!(context.get::<ConfigTree>().is_some());
}

#[test]
fn run_skips_etc_when_empty_config_is_true() {
    let source = TempDir::new().unwrap();
    create_dir_all(source.path().join("usr")).unwrap();
    write(source.path().join("usr/tool"), b"binary").unwrap();
    create_dir_all(source.path().join("etc")).unwrap();
    write(source.path().join("etc/conf"), b"config").unwrap();

    let (mut context, _target_scratch) = context_with(source.path().to_path_buf(), true);
    let cancel = CancelToken::new();
    let progress = ProgressEventBuilder::new(0);

    ImportTreesStage.run(&mut context, &cancel, progress).unwrap();

    let config_paths = context.get::<ImportedConfigPaths>().unwrap();
    assert!(config_paths.0.is_empty());

    let prefix_paths = context.get::<ImportedPrefixPaths>().unwrap();
    assert_eq!(prefix_paths.0, vec![PathBuf::from("tool")]);
}

#[test]
fn run_handles_source_with_neither_usr_nor_etc() {
    let source = TempDir::new().unwrap();

    let (mut context, _target_scratch) = context_with(source.path().to_path_buf(), false);
    let cancel = CancelToken::new();
    let progress = ProgressEventBuilder::new(0);

    let (_, result, _guard) = ImportTreesStage.run(&mut context, &cancel, progress).unwrap();

    assert!(matches!(result, StageResult::Advance));
    assert!(context.get::<ImportedPrefixPaths>().unwrap().0.is_empty());
    assert!(context.get::<ImportedConfigPaths>().unwrap().0.is_empty());
}
