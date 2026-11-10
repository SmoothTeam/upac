// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_types::CancelToken;
use upac_types::error::ErrorKind;

use upac_deploy::Sysroot;

use upac_macro::stage;

use upac_orchestrator::context::Context;
use upac_orchestrator::stage::Stage;

use super::{RequestedPinned, RequestedPrefixDigest};

pub struct SetPinnedStage;

#[stage]
impl Stage<ErrorKind> for SetPinnedStage {
    fn run(
        &self, context: &mut Context, _cancel: &CancelToken, _progress: &dyn Fn(Option<&str>, u64, u64),
    ) -> Result<(), ErrorKind> {
        let sysroot = context.get::<Sysroot>()?;
        let pinned = context.get::<RequestedPinned>()?.0;

        let mut prefix = sysroot.prefix(&context.get::<RequestedPrefixDigest>()?.0)?;
        if prefix.pinned() == pinned {
            return Ok(());
        }

        prefix.set_pinned(pinned);
        sysroot.save_prefix(&prefix)?;

        Ok(())
    }
}
