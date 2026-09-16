// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::fs::{create_dir_all, write};

use tempfile::TempDir;

use upac::orchestrator::context::Context;
use upac::orchestrator::stage::{Stage, StageResult};

use upac_abi::hook::CancelToken;

use upac_types::TmpPath;
use upac_types::hook::ProgressEventBuilder;

use super::super::{ConfigState, PrefixTree, ResolvedSourceDir, SetupProgress, UnpackState};
use super::EnumeratePackagesStage;

#[test]
fn run_lists_only_files_and_initializes_pipeline_state() {
    let source = TempDir::new().unwrap();
    write(source.path().join("a.pkg.tar.zst"), b"a").unwrap();
    write(source.path().join("b.pkg.tar.zst"), b"b").unwrap();
    create_dir_all(source.path().join("not-a-package")).unwrap();

    let mut context = Context::new();
    context.put(ResolvedSourceDir(source.path().to_path_buf()));

    let cancel = CancelToken::new();
    let progress = ProgressEventBuilder::new(0);

    let (_, result, _guard) = EnumeratePackagesStage.run(&mut context, &cancel, progress).unwrap();

    assert!(matches!(result, StageResult::Advance));

    let setup_progress = context.get::<SetupProgress>().unwrap();
    assert_eq!(setup_progress.total, 2);
    assert!(setup_progress.pending.is_empty());

    let unpack_state = context.get::<UnpackState>().unwrap();
    assert_eq!(unpack_state.pending_paths.len(), 2);

    assert!(context.get::<TmpPath>().is_some());
    assert!(context.get::<ConfigState>().is_some());
    assert!(context.get::<PrefixTree>().is_some());
}

#[test]
fn run_with_empty_directory_sets_total_to_zero() {
    let source = TempDir::new().unwrap();

    let mut context = Context::new();
    context.put(ResolvedSourceDir(source.path().to_path_buf()));

    let cancel = CancelToken::new();
    let progress = ProgressEventBuilder::new(0);

    EnumeratePackagesStage.run(&mut context, &cancel, progress).unwrap();

    let setup_progress = context.get::<SetupProgress>().unwrap();
    assert_eq!(setup_progress.total, 0);

    let unpack_state = context.get::<UnpackState>().unwrap();
    assert!(unpack_state.pending_paths.is_empty());
}

#[test]
fn run_fails_when_source_dir_missing_from_context() {
    let mut context = Context::new();

    let cancel = CancelToken::new();
    let progress = ProgressEventBuilder::new(0);

    let result = EnumeratePackagesStage.run(&mut context, &cancel, progress);

    assert!(result.is_err());
}
