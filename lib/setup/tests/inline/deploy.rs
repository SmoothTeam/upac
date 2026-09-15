// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use composefs::fsverity::FsVerityHashValue;

use tempfile::TempDir;

use upac::composefs::repository::ObjectID;
use upac::database::record::DeployRecord;
use upac::orchestrator::context::Context;
use upac::orchestrator::stage::{Stage, StageResult};

use upac_abi::hook::{CancelToken, ProgressEventBuilder};

use crate::target::TargetSysroot;

use super::super::Pinned;
use super::super::database::{ConfigDigest, PrefixDigest};
use super::WriteDeployRecordStage;

#[test]
fn run_writes_a_deploy_record_readable_back_from_disk() {
    let scratch = TempDir::new().unwrap();
    let target = TargetSysroot::for_testing(scratch.path().to_path_buf()).unwrap();

    let mut context = Context::new();
    context.put(target);
    context.put(Pinned(true));
    context.put(PrefixDigest(ObjectID::EMPTY));
    context.put(ConfigDigest(ObjectID::EMPTY));

    let cancel = CancelToken::new();
    let progress = ProgressEventBuilder::new(0);

    let (_, result, _guard) = WriteDeployRecordStage.run(&mut context, &cancel, progress).unwrap();

    assert!(matches!(result, StageResult::Advance));

    let target = context.get::<TargetSysroot>().unwrap();
    let deploy_dir = target.deploy_dir(&ObjectID::EMPTY.to_hex());

    let record = DeployRecord::read(&deploy_dir).unwrap();
    assert_eq!(record.prefix_digest, ObjectID::EMPTY.to_hex());
    assert_eq!(record.subject, "genesis");
    assert_eq!(record.message, None);
    assert!(record.pinned);
    assert!(record.config_history.is_empty());
    assert_eq!(record.working_config, ObjectID::EMPTY.to_hex());
}

#[test]
fn run_fails_when_target_missing_from_context() {
    let mut context = Context::new();
    context.put(Pinned(false));
    context.put(PrefixDigest(ObjectID::EMPTY));
    context.put(ConfigDigest(ObjectID::EMPTY));

    let cancel = CancelToken::new();
    let progress = ProgressEventBuilder::new(0);

    let result = WriteDeployRecordStage.run(&mut context, &cancel, progress);

    assert!(result.is_err());
}
