// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use nix::errno::Errno;

use upac::lock::{Lock, LockError};

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

// Single test, not split across multiple `#[test]` fns: `Lock::acquire` binds a fixed abstract
// unix socket address shared process-wide, so concurrent tests acquiring it in parallel would
// race each other. Sequencing acquire/busy/drop/reacquire inside one function keeps it self
// contained.
#[test]
fn lock_prevents_concurrent_acquisition_and_releases_on_drop() {
    let Ok(first) = Lock::acquire() else {
        panic!("first acquisition should succeed when uncontended");
    };

    let second = Lock::acquire();
    assert!(matches!(second, Err(LockError::Busy)));

    drop(first);

    assert!(Lock::acquire().is_ok());
}
