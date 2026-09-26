// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::ptr::null_mut;

use upac_abi::error::{CError, ErrorDomain, ErrorKind};

use upac_types::error::{Error, write_abi_error};
use upac_types::state::mutated::RollbackStateId;

#[test]
fn new_takes_the_domain_from_the_state_type() {
    let error = Error::new(RollbackStateId::Setup, ErrorKind::NotFound);

    assert_eq!(error.domain, ErrorDomain::Rollback);
    assert_eq!(error.state, RollbackStateId::Setup as u32);
    assert_eq!(error.kind, ErrorKind::NotFound);
}

#[test]
fn catch_passes_a_successful_value_through() {
    let result = Error::catch(|| Ok::<_, (RollbackStateId, ErrorKind)>(7));

    assert_eq!(result, Ok(7));
}

#[test]
fn catch_turns_a_stage_failure_into_an_error() {
    let result = Error::catch(|| Err::<(), _>((RollbackStateId::Setup, ErrorKind::WriteFailed)));

    assert_eq!(result, Err(Error::new(RollbackStateId::Setup, ErrorKind::WriteFailed)));
}

#[test]
fn catch_turns_a_panic_into_an_unexpected_validation_error() {
    let result = Error::catch(|| -> Result<(), (RollbackStateId, ErrorKind)> { panic!("stage panicked") });

    assert_eq!(result, Err(Error::new(RollbackStateId::Setup, ErrorKind::Unexpected)));
}

#[test]
fn converts_to_c_and_back_without_loss() {
    let error = Error::new(RollbackStateId::Setup, ErrorKind::NoSpaceLeft);

    let c_error = CError::from(error);

    assert_eq!(Error::try_from(&c_error), Ok(error));
}

#[test]
fn check_is_ok_for_a_zero_code() {
    assert_eq!(Error::check(0, &CError::default()), Ok(()));
}

#[test]
fn check_returns_the_written_error_for_a_nonzero_code() {
    let error = Error::new(RollbackStateId::Setup, ErrorKind::PermissionDenied);

    assert_eq!(Error::check(-1, &CError::from(error)), Err(error));
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
fn write_abi_error_fills_the_out_pointer_and_returns_minus_one() {
    let error = Error::new(RollbackStateId::Setup, ErrorKind::Cancelled);
    let mut c_error = CError::default();

    let code = unsafe { write_abi_error(&mut c_error, error) };

    assert_eq!(code, -1);
    assert_eq!(Error::try_from(&c_error), Ok(error));
}

#[test]
fn write_abi_error_tolerates_a_null_out_pointer() {
    let error = Error::new(RollbackStateId::Setup, ErrorKind::Cancelled);

    assert_eq!(unsafe { write_abi_error(null_mut(), error) }, -1);
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
