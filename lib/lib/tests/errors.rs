// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::io::ErrorKind as IoErrorKind;

use upac::boot::error::BootError;
use upac::composefs::error::RepoError;
use upac::database::error::{DatabaseError, DeployRecordError, DeployRecordsError};
use upac::deploy::error::SysrootError;
use upac::errors::CommonError;
use upac::lock::LockError;
use upac::plugin::boot::error::BootPluginError;
use upac::plugin::decoder::error::DecoderError;
use upac::scripts::error::HookError;

use upac_abi::error::ErrorKind;

#[test]
fn common_error_no_payload_variants_map_directly() {
    let cases = [
        (CommonError::OutOfMemory, ErrorKind::OutOfMemory),
        (CommonError::Cancelled, ErrorKind::Cancelled),
        (CommonError::AccessDenied, ErrorKind::PermissionDenied),
        (CommonError::StageNotFound, ErrorKind::Unexpected),
        (CommonError::StagePanicked, ErrorKind::Unexpected),
        (CommonError::MissingResult, ErrorKind::Unexpected),
        (CommonError::PipelineInvalid, ErrorKind::Unexpected),
        (CommonError::RuntimeInit(IoErrorKind::Other), ErrorKind::Unexpected),
    ];

    for (error, expected) in cases {
        assert_eq!(ErrorKind::from(error), expected);
    }
}

#[test]
fn common_error_hook_variant_delegates_to_the_inner_conversion() {
    let error = HookError::Parse;

    assert_eq!(CommonError::from(error.clone()), CommonError::Hook(error.clone()));
    assert_eq!(
        ErrorKind::from(CommonError::Hook(error.clone())),
        ErrorKind::from(error)
    );
}

#[test]
fn common_error_decoder_variant_delegates_to_the_inner_conversion() {
    let error = DecoderError::Manifest;

    assert_eq!(CommonError::from(error.clone()), CommonError::Decoder(error.clone()));
    assert_eq!(
        ErrorKind::from(CommonError::Decoder(error.clone())),
        ErrorKind::from(error)
    );
}

#[test]
fn common_error_repo_variant_delegates_to_the_inner_conversion() {
    let error = RepoError::NotFound;

    assert_eq!(CommonError::from(error), CommonError::Repo(error));
    assert_eq!(ErrorKind::from(CommonError::Repo(error)), ErrorKind::from(error));
}

#[test]
fn common_error_database_variant_delegates_to_the_inner_conversion() {
    let error = DatabaseError::WriteError;

    assert_eq!(CommonError::from(error), CommonError::Database(error));
    assert_eq!(ErrorKind::from(CommonError::Database(error)), ErrorKind::from(error));
}

#[test]
fn common_error_sysroot_variant_delegates_to_the_inner_conversion() {
    let error = SysrootError::RootDeviceNotFound;

    assert_eq!(CommonError::from(error), CommonError::Sysroot(error));
    assert_eq!(ErrorKind::from(CommonError::Sysroot(error)), ErrorKind::from(error));
}

#[test]
fn common_error_lock_variant_delegates_to_the_inner_conversion() {
    let error = LockError::Busy;

    assert_eq!(CommonError::from(error), CommonError::Lock(error));
    assert_eq!(ErrorKind::from(CommonError::Lock(error)), ErrorKind::from(error));
}

#[test]
fn common_error_deploy_record_variant_delegates_to_the_inner_conversion() {
    let error = DeployRecordError::WriteFailed;

    assert_eq!(CommonError::from(error), CommonError::DeployRecord(error));
    assert_eq!(
        ErrorKind::from(CommonError::DeployRecord(error)),
        ErrorKind::from(error)
    );
}

#[test]
fn common_error_boot_variant_delegates_to_the_inner_conversion() {
    let error = BootError::NoBootResource;

    assert_eq!(CommonError::from(error.clone()), CommonError::Boot(error.clone()));
    assert_eq!(
        ErrorKind::from(CommonError::Boot(error.clone())),
        ErrorKind::from(error)
    );
}

#[test]
fn common_error_boot_plugin_variant_delegates_to_the_inner_conversion() {
    let error = BootPluginError::NoClaimant;

    assert_eq!(CommonError::from(error.clone()), CommonError::BootPlugin(error.clone()));
    assert_eq!(
        ErrorKind::from(CommonError::BootPlugin(error.clone())),
        ErrorKind::from(error)
    );
}

#[test]
fn common_error_from_deploy_records_error_unwraps_the_sysroot_variant() {
    let error = DeployRecordsError::Sysroot(SysrootError::RepoDirNotFound);

    assert_eq!(
        CommonError::from(error),
        CommonError::Sysroot(SysrootError::RepoDirNotFound)
    );
}

#[test]
fn common_error_from_deploy_records_error_unwraps_the_deploy_record_variant() {
    let error = DeployRecordsError::DeployRecord(DeployRecordError::NotFound);

    assert_eq!(
        CommonError::from(error),
        CommonError::DeployRecord(DeployRecordError::NotFound)
    );
}
