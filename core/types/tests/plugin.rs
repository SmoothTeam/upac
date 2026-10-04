// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_types::booter::BootError;
use upac_types::decoder::DecodeError;
use upac_types::plugin::{plugin_result, plugin_status};

#[test]
fn success_round_trips() {
    let status = plugin_status::<DecodeError>(Ok(()));

    assert_eq!(plugin_result::<DecodeError>(status), Ok(()));
}

#[test]
fn a_decode_error_round_trips_through_its_status() {
    let status = plugin_status(Err(DecodeError::ChecksumMismatch));

    assert_eq!(plugin_result(status), Err(Some(DecodeError::ChecksumMismatch)));
}

#[test]
fn a_boot_error_round_trips_through_its_status() {
    let status = plugin_status(Err(BootError::NoFreeBootId));

    assert_eq!(plugin_result(status), Err(Some(BootError::NoFreeBootId)));
}

#[test]
fn an_error_status_is_never_the_success_status() {
    assert_ne!(
        plugin_status(Err(DecodeError::InvalidRequest)),
        plugin_status::<DecodeError>(Ok(()))
    );
    assert_ne!(
        plugin_status(Err(BootError::InvalidRequest)),
        plugin_status::<BootError>(Ok(()))
    );
}

#[test]
fn an_unknown_status_is_reported_as_unknown() {
    assert_eq!(plugin_result::<DecodeError>(-1), Err(None));
    assert_eq!(plugin_result::<BootError>(99), Err(None));
}
