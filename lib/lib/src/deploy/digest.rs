// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later

use linux_kernel_cmdline::utf8::CmdlineOwned;

use crate::deploy::error::SysrootError;
use crate::layout::deployment::PREFIX_DIGEST_CMDLINE_PARAM;

pub fn current_prefix_digest() -> Result<String, SysrootError> {
    let cmdline = CmdlineOwned::from_proc()?;
    let digest = cmdline.require_value_of(PREFIX_DIGEST_CMDLINE_PARAM)?;

    Ok(digest.to_owned())
}
