// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later

use upac::deploy::digest::current_prefix_digest;
use upac::deploy::error::SysrootError;

#[test]
fn current_prefix_digest_fails_when_param_is_absent_from_cmdline() {
    let result = current_prefix_digest();

    assert!(matches!(result, Err(SysrootError::CurrentPrefixDigestNotFound)));
}
