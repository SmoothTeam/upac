// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_types::CancelToken;
use upac_types::error::ErrorKind;
use upac_types::request::format::FsKind;

use upac_macro::stage;

use upac_orchestrator::context::Context;
use upac_orchestrator::stage::Stage;

use super::{RequestedDevice, RequestedFilesystem, RequireEsp};

use crate::gpt::Partition;

pub struct VerifyStage;

#[stage]
impl Stage<ErrorKind> for VerifyStage {
    fn run(
        &self, context: &mut Context, _cancel: &CancelToken, _progress: &dyn Fn(Option<&str>, u64, u64),
    ) -> Result<(), ErrorKind> {
        if !context.get::<RequireEsp>()?.0 {
            return Ok(());
        }

        if context.get::<RequestedFilesystem>()?.fs_kind != FsKind::Vfat {
            return Err(ErrorKind::InvalidEntry);
        }

        if !Partition::open(&context.get::<RequestedDevice>()?.0)?.is_esp() {
            return Err(ErrorKind::WrongPartitionType);
        }

        Ok(())
    }
}
