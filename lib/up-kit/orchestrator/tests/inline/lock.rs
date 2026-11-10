// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use super::{Lock, LockError};

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
