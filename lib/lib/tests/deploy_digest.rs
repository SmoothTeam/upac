// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 JustPav
//
// SPDX-License-Identifier: LGPL-3.0-or-later

use upac::deploy::digest::current_usr_digest;
use upac::deploy::error::SysrootError;

#[test]
fn current_usr_digest_fails_when_param_is_absent_from_cmdline() {
    let result = current_usr_digest();

    assert!(matches!(result, Err(SysrootError::CurrentUsrDigestNotFound)));
}
