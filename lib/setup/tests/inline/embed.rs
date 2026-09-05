// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use composefs::generic_tree::Stat;
use composefs::repository::ImportContext;
use composefs::tree::FileSystem;

use tempfile::TempDir;

use upac::database::{InMemory, MemoryDatabase};
use upac::orchestrator::Context;
use upac::orchestrator::stage::{Stage, StageResult};

use upac_abi::hook::{CancelToken, ProgressEventBuilder};

use crate::target::TargetSysroot;
use crate::types::{ConfigDigest, ConfigTree, GenesisDatabase, PrefixDigest, PrefixTree};

use super::EmbedDatabaseStage;

#[test]
fn run_commits_both_trees_and_puts_digests() {
    let scratch = TempDir::new().unwrap();
    let target = TargetSysroot::for_testing(scratch.path().to_path_buf()).unwrap();

    let mut context = Context::new();
    context.put(target);
    context.put(PrefixTree(FileSystem::new(Stat::uninitialized())));
    context.put(ConfigTree(FileSystem::new(Stat::uninitialized())));
    context.put(GenesisDatabase(MemoryDatabase::new_in_memory().unwrap()));
    context.put(ImportContext::default());

    let cancel = CancelToken::new();
    let progress = ProgressEventBuilder::new(0);

    let (_, result, _guard) = EmbedDatabaseStage.run(&mut context, &cancel, progress).unwrap();

    assert!(matches!(result, StageResult::Advance));
    assert!(context.get::<PrefixDigest>().is_some());
    assert!(context.get::<ConfigDigest>().is_some());
}

#[test]
fn run_fails_when_database_missing_from_context() {
    let scratch = TempDir::new().unwrap();
    let target = TargetSysroot::for_testing(scratch.path().to_path_buf()).unwrap();

    let mut context = Context::new();
    context.put(target);
    context.put(PrefixTree(FileSystem::new(Stat::uninitialized())));
    context.put(ConfigTree(FileSystem::new(Stat::uninitialized())));
    context.put(ImportContext::default());

    let cancel = CancelToken::new();
    let progress = ProgressEventBuilder::new(0);

    let result = EmbedDatabaseStage.run(&mut context, &cancel, progress);

    assert!(result.is_err());
}
