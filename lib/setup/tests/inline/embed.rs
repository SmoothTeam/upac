// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use composefs::generic_tree::Stat;
use composefs::repository::ImportContext;
use composefs::tree::FileSystem;

use tempfile::TempDir;

use upac::composefs::file::FileHandle;
use upac::database::{InMemory, MemoryDatabase};
use upac::layout::database::DATABASE_PATH;
use upac::orchestrator::context::Context;
use upac::orchestrator::stage::{Stage, StageResult};

use upac_abi::hook::CancelToken;

use upac_types::hook::ProgressEventBuilder;

use crate::target::TargetSysroot;

use super::super::{PackageDatabase, PackageScratch, PrefixTree};
use super::EmbedDatabaseStage;

#[test]
fn run_inserts_the_database_into_the_prefix_tree() {
    let scratch = TempDir::new().unwrap();
    let target = TargetSysroot::for_testing(scratch.path().to_path_buf()).unwrap();

    let mut context = Context::new();
    context.put(PackageScratch(TempDir::new().unwrap()));
    context.put(target);
    context.put(PrefixTree(FileSystem::new(Stat::uninitialized())));
    context.put(PackageDatabase(MemoryDatabase::new_in_memory().unwrap()));
    context.put(ImportContext::default());

    let cancel = CancelToken::new();
    let progress = ProgressEventBuilder::new(0);

    let (_, result, _guard) = EmbedDatabaseStage.run(&mut context, &cancel, progress).unwrap();

    assert!(matches!(result, StageResult::Advance));
    assert!(context.get::<PackageDatabase>().is_none());

    let prefix_tree = context.get::<PrefixTree>().unwrap();
    assert!(FileHandle::new(DATABASE_PATH).stat_in_tree(prefix_tree).is_ok());
}

#[test]
fn run_fails_when_database_missing_from_context() {
    let scratch = TempDir::new().unwrap();
    let target = TargetSysroot::for_testing(scratch.path().to_path_buf()).unwrap();

    let mut context = Context::new();
    context.put(target);
    context.put(PrefixTree(FileSystem::new(Stat::uninitialized())));
    context.put(ImportContext::default());

    let cancel = CancelToken::new();
    let progress = ProgressEventBuilder::new(0);

    let result = EmbedDatabaseStage.run(&mut context, &cancel, progress);

    assert!(result.is_err());
}
