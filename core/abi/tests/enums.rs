// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_abi::error::{ErrorDomain, ErrorKind};
use upac_abi::hook::HookAck;
use upac_abi::plugin::BootResourceKind;
use upac_abi::request::format::FsKind;
use upac_abi::response::entry::PackageDiffKind;

#[test]
fn a_known_value_round_trips_through_its_raw_integer() {
    assert_eq!(FsKind::try_from(u8::from(FsKind::Xfs)), Ok(FsKind::Xfs));
    assert_eq!(
        PackageDiffKind::try_from(u8::from(PackageDiffKind::FilesChanged)),
        Ok(PackageDiffKind::FilesChanged)
    );
    assert_eq!(
        ErrorKind::try_from(u32::from(ErrorKind::ToolFailed)),
        Ok(ErrorKind::ToolFailed)
    );
}

#[test]
fn an_out_of_range_value_is_an_invalid_entry() {
    assert_eq!(FsKind::try_from(4), Err(ErrorKind::InvalidEntry));
    assert_eq!(BootResourceKind::try_from(u8::MAX), Err(ErrorKind::InvalidEntry));
    assert_eq!(HookAck::try_from(2), Err(ErrorKind::InvalidEntry));
    assert_eq!(ErrorDomain::try_from(u32::MAX), Err(ErrorKind::InvalidEntry));
}

#[test]
fn the_unknown_domain_is_zero() {
    assert_eq!(u32::from(ErrorDomain::Unknown), 0);
}

#[test]
fn zero_is_success_and_never_an_error_kind() {
    assert_eq!(ErrorKind::try_from(0), Err(ErrorKind::InvalidEntry));
    assert_eq!(u32::from(ErrorKind::Unexpected), 1);
}
