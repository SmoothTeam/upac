// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::path::PathBuf;

use upac_types::error::ErrorKind;

use super::root_partition;

#[test]
fn an_explicit_deploy_device_is_taken_as_is() {
    assert_eq!(root_partition(None, Some("/dev/vda2")), Ok(PathBuf::from("/dev/vda2")));
}

#[test]
fn a_disk_and_a_device_together_or_neither_are_rejected() {
    assert_eq!(root_partition(None, None), Err(ErrorKind::InvalidEntry));
    assert_eq!(
        root_partition(Some("/dev/vda"), Some("/dev/vda2")),
        Err(ErrorKind::InvalidEntry)
    );
}
