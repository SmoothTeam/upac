// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac::database::meta::MetaStore;
use upac::orchestrator::Context;
use upac::orchestrator::stage::{Stage, StageResult};

use upac_abi::hook::{CancelToken, ProgressEventBuilder};

use upac_types::PackageMeta;

use crate::types::{GenesisDatabase, PackageUuid};

use super::CreateDatabaseStage;

#[test]
fn run_inserts_package_meta_and_puts_database_and_uuid() {
    let mut context = Context::new();
    context.put(PackageMeta {
        name: "test-pkg".to_owned(),
        ..PackageMeta::default()
    });

    let cancel = CancelToken::new();
    let progress = ProgressEventBuilder::new(0);

    let (_, result, _) = CreateDatabaseStage.run(&mut context, &cancel, progress).unwrap();

    assert!(matches!(result, StageResult::Advance));

    let database = context
        .take::<GenesisDatabase>()
        .expect("stage should put GenesisDatabase");
    let uuid = context.get::<PackageUuid>().expect("stage should put PackageUuid");

    let meta = database
        .0
        .get_package_meta(uuid.0)
        .unwrap()
        .expect("meta should be retrievable by the uuid the stage produced");
    assert_eq!(meta.name, "test-pkg");
}

#[test]
fn run_fails_when_package_meta_missing_from_context() {
    let mut context = Context::new();
    let cancel = CancelToken::new();
    let progress = ProgressEventBuilder::new(0);

    let result = CreateDatabaseStage.run(&mut context, &cancel, progress);

    assert!(result.is_err());
}
