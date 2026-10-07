// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_types::CancelToken;
use upac_types::error::ErrorKind;

use upac_macro::stage;

use upac_orchestrator::context::Context;
use upac_orchestrator::stage::Stage;

use super::{ForceWipe, RequestedDevice};

use crate::wipe::WipeTarget;

pub struct WipeStage;

#[stage]
impl Stage<ErrorKind> for WipeStage {
    fn run(
        &self, context: &mut Context, _cancel: &CancelToken, _progress: &dyn Fn(Option<&str>, u64, u64),
    ) -> Result<(), ErrorKind> {
        let force_wipe = context.get::<ForceWipe>()?.0;

        WipeTarget {
            device_path: &context.get::<RequestedDevice>()?.0,
        }
        .wipe_or_refuse(force_wipe)
    }
}
