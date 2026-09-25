// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use nix::errno::Errno;

use upac_orchestrator::lock::LockError;

use upac_abi::error::ErrorKind;

#[test]
fn lock_error_from_errno_maps_known_errnos() {
    assert_eq!(LockError::from(Errno::EADDRINUSE), LockError::Busy);
    assert_eq!(LockError::from(Errno::EROFS), LockError::ReadOnly);
    assert_eq!(LockError::from(Errno::EPERM), LockError::Denied);
    assert_eq!(LockError::from(Errno::EACCES), LockError::Denied);
    assert_eq!(LockError::from(Errno::ENOENT), LockError::PathMissing);
    assert_eq!(LockError::from(Errno::EIO), LockError::Unexpected(Errno::EIO));
}

#[test]
fn lock_error_to_error_kind_mapping() {
    assert_eq!(ErrorKind::from(LockError::Busy), ErrorKind::Unexpected);
    assert_eq!(ErrorKind::from(LockError::ReadOnly), ErrorKind::PermissionDenied);
    assert_eq!(ErrorKind::from(LockError::Denied), ErrorKind::PermissionDenied);
    assert_eq!(ErrorKind::from(LockError::PathMissing), ErrorKind::InvalidPath);
    assert_eq!(
        ErrorKind::from(LockError::Unexpected(Errno::EIO)),
        ErrorKind::Unexpected
    );
}
