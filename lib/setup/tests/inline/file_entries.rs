// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::path::PathBuf;

use upac::database::files::FileStore;
use upac::database::meta::MetaStoreMut;
use upac::database::{InMemory, MemoryDatabase};
use upac::orchestrator::Context;
use upac::orchestrator::stage::{Stage, StageResult};

use upac_abi::hook::{CancelToken, ProgressEventBuilder};

use upac_types::{FileEntryScope, PackageMeta};

use crate::types::{GenesisDatabase, ImportedConfigPaths, ImportedPrefixPaths, PackageUuid};

use super::InsertFileEntryStage;

fn context_with(prefix_paths: Vec<PathBuf>, config_paths: Vec<PathBuf>) -> Context {
    let mut database = MemoryDatabase::new_in_memory().unwrap();
    let uuid = database
        .insert_package_meta(&PackageMeta {
            name: "test-pkg".to_owned(),
            ..PackageMeta::default()
        })
        .unwrap();

    let mut context = Context::new();
    context.put(ImportedPrefixPaths(prefix_paths));
    context.put(ImportedConfigPaths(config_paths));
    context.put(GenesisDatabase(database));
    context.put(PackageUuid(uuid));
    context
}

#[test]
fn run_inserts_one_entry_per_call_and_repeats_until_all_paths_are_recorded() {
    let mut context = context_with(
        vec![PathBuf::from("usr/bin/a"), PathBuf::from("usr/bin/b")],
        vec![PathBuf::from("etc/conf")],
    );
    let cancel = CancelToken::new();

    let (_, first, _) = InsertFileEntryStage
        .run(&mut context, &cancel, ProgressEventBuilder::new(0))
        .unwrap();
    assert!(matches!(first, StageResult::Repeat));

    let (_, second, _) = InsertFileEntryStage
        .run(&mut context, &cancel, ProgressEventBuilder::new(0))
        .unwrap();
    assert!(matches!(second, StageResult::Repeat));

    let (_, third, _) = InsertFileEntryStage
        .run(&mut context, &cancel, ProgressEventBuilder::new(0))
        .unwrap();
    assert!(matches!(third, StageResult::Advance));

    let uuid = context.get::<PackageUuid>().unwrap();
    let database = context.get::<GenesisDatabase>().unwrap();
    let mut files = database.0.list_package_files(uuid.0).unwrap();
    files.sort_by(|a, b| a.path.cmp(&b.path));

    assert_eq!(files.len(), 3);
    assert_eq!(files[0].path, "etc/conf");
    assert_eq!(files[0].scope, FileEntryScope::Config);
    assert_eq!(files[1].path, "usr/bin/a");
    assert_eq!(files[1].scope, FileEntryScope::Prefix);
    assert_eq!(files[2].path, "usr/bin/b");
    assert_eq!(files[2].scope, FileEntryScope::Prefix);
}

#[test]
fn run_advances_immediately_when_both_queues_are_empty() {
    let mut context = context_with(Vec::new(), Vec::new());
    let cancel = CancelToken::new();

    let (_, result, _) = InsertFileEntryStage
        .run(&mut context, &cancel, ProgressEventBuilder::new(0))
        .unwrap();

    assert!(matches!(result, StageResult::Advance));

    let uuid = context.get::<PackageUuid>().unwrap();
    let database = context.get::<GenesisDatabase>().unwrap();
    assert!(database.0.list_package_files(uuid.0).unwrap().is_empty());
}
