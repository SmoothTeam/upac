// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::ptr::null_mut;

use upac_abi::error::CError;

use upac_types::error::{
    Error, ErrorDomain, ErrorKind, export_mutated_command, export_mutated_command_with_response,
    export_unmutated_command,
};
use upac_types::state::mutated::RollbackStateId;

struct FakeCRequest {
    valid: bool,
}

struct FakeRequest;

impl TryFrom<&FakeCRequest> for FakeRequest {
    type Error = ErrorKind;

    fn try_from(request: &FakeCRequest) -> Result<Self, ErrorKind> {
        if request.valid {
            Ok(FakeRequest)
        } else {
            Err(ErrorKind::InvalidEntry)
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
struct FakeCResponse(u32);

impl From<u32> for FakeCResponse {
    fn from(value: u32) -> Self {
        FakeCResponse(value)
    }
}

const VALID: FakeCRequest = FakeCRequest { valid: true };

#[test]
fn new_takes_the_domain_from_the_state_type() {
    let error = Error::new(RollbackStateId::Setup, ErrorKind::NotFound);

    assert_eq!(error.domain, ErrorDomain::Rollback);
    assert_eq!(error.state, RollbackStateId::Setup as u32);
    assert_eq!(error.kind, ErrorKind::NotFound);
}

#[test]
fn converts_to_c_and_back_without_loss() {
    let error = Error::new(RollbackStateId::Setup, ErrorKind::NoSpaceLeft);

    let c_error = CError::from(error.clone());

    assert_eq!(Error::try_from(&c_error), Ok(error));
}

#[test]
fn check_is_ok_for_a_zero_code() {
    assert_eq!(Error::check(0, &CError::default()), Ok(()));
}

#[test]
fn check_returns_the_written_error_for_a_nonzero_code() {
    let error = Error::new(RollbackStateId::Setup, ErrorKind::PermissionDenied);

    assert_eq!(Error::check(-1, &CError::from(error.clone())), Err(error));
}

#[test]
fn check_reports_an_abi_mismatch_for_a_foreign_struct_size() {
    let c_error = CError {
        struct_size: 0,
        ..CError::from(Error::new(RollbackStateId::Setup, ErrorKind::NotFound))
    };

    let error = Error::check(-1, &c_error).unwrap_err();

    assert_eq!(error.domain, ErrorDomain::Rollback);
    assert_eq!(error.kind, ErrorKind::AbiMismatch);
}

#[test]
fn check_falls_back_to_the_unknown_domain_for_an_out_of_range_value() {
    let c_error = CError {
        domain: u32::MAX,
        ..CError::from(Error::new(RollbackStateId::Setup, ErrorKind::NotFound))
    };

    let error = Error::check(-1, &c_error).unwrap_err();

    assert_eq!(error.domain, ErrorDomain::Unknown);
    assert_eq!(error.kind, ErrorKind::InvalidEntry);
}

#[test]
fn a_successful_mutated_command_returns_zero_and_leaves_the_error_untouched() {
    let mut c_error = CError::default();

    let code = unsafe {
        export_mutated_command(&VALID, &mut c_error, |_: FakeRequest| {
            Ok::<_, (RollbackStateId, ErrorKind, Option<String>)>(())
        })
    };

    assert_eq!(code, 0);
    assert_eq!(c_error.kind, CError::default().kind);
}

#[test]
fn a_failed_stage_is_written_with_its_state() {
    let mut c_error = CError::default();

    let code = unsafe {
        export_mutated_command(&VALID, &mut c_error, |_: FakeRequest| {
            Err((RollbackStateId::Setup, ErrorKind::WriteFailed, None))
        })
    };

    assert_eq!(code, -1);
    assert_eq!(
        Error::try_from(&c_error),
        Ok(Error::new(RollbackStateId::Setup, ErrorKind::WriteFailed))
    );
}

#[test]
fn a_panic_becomes_an_unexpected_validation_error() {
    let mut c_error = CError::default();

    let code = unsafe {
        export_mutated_command(
            &VALID,
            &mut c_error,
            |_: FakeRequest| -> Result<(), (RollbackStateId, ErrorKind, Option<String>)> { panic!("stage panicked") },
        )
    };

    assert_eq!(code, -1);
    assert_eq!(
        Error::try_from(&c_error),
        Ok(Error::new(RollbackStateId::Setup, ErrorKind::Unexpected))
    );
}

#[test]
fn an_invalid_request_is_a_validation_error_and_run_is_never_called() {
    let mut c_error = CError::default();

    let code = unsafe {
        export_mutated_command(
            &FakeCRequest { valid: false },
            &mut c_error,
            |_: FakeRequest| -> Result<(), (RollbackStateId, ErrorKind, Option<String>)> {
                panic!("run must not be called")
            },
        )
    };

    assert_eq!(code, -1);
    assert_eq!(
        Error::try_from(&c_error),
        Ok(Error::new(RollbackStateId::Setup, ErrorKind::InvalidEntry))
    );
}

#[test]
fn a_null_error_pointer_is_tolerated() {
    let code = unsafe {
        export_mutated_command(&VALID, null_mut(), |_: FakeRequest| {
            Err((RollbackStateId::Setup, ErrorKind::Cancelled, None))
        })
    };

    assert_eq!(code, -1);
}

#[test]
fn an_unmutated_command_writes_its_response() {
    let mut response = FakeCResponse(0);
    let mut c_error = CError::default();

    let code = unsafe {
        export_unmutated_command(&VALID, &mut response, &mut c_error, |_: FakeRequest| {
            Ok::<_, (RollbackStateId, ErrorKind, Option<String>)>(7u32)
        })
    };

    assert_eq!(code, 0);
    assert_eq!(response, FakeCResponse(7));
}

#[test]
fn an_unmutated_command_tolerates_a_null_response_pointer() {
    let code = unsafe {
        export_unmutated_command(&VALID, null_mut::<FakeCResponse>(), null_mut(), |_: FakeRequest| {
            Ok::<_, (RollbackStateId, ErrorKind, Option<String>)>(7u32)
        })
    };

    assert_eq!(code, 0);
}

#[test]
fn a_mutated_command_with_a_response_writes_it() {
    let mut response = FakeCResponse(0);
    let mut c_error = CError::default();

    let code = unsafe {
        export_mutated_command_with_response(&VALID, &mut response, &mut c_error, |_: FakeRequest| {
            Ok::<_, (RollbackStateId, ErrorKind, Option<String>)>(7u32)
        })
    };

    assert_eq!(code, 0);
    assert_eq!(response, FakeCResponse(7));
}

#[test]
fn a_failure_subject_travels_through_the_c_error() {
    let mut c_error = CError::default();

    let code = unsafe {
        export_mutated_command(&VALID, &mut c_error, |_: FakeRequest| {
            Err((
                RollbackStateId::Setup,
                ErrorKind::NotFound,
                Some("system/sysroot".to_owned()),
            ))
        })
    };

    assert_eq!(code, -1);
    assert_eq!(
        Error::try_from(&c_error).map(|error| error.subject),
        Ok(Some("system/sysroot".to_owned()))
    );
    unsafe { c_error.free() };
}
