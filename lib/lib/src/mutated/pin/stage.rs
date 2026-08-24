// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later

use std::mem::replace;

use upac_abi::hook::{CancelToken, ProgressEventBuilder};

use crate::database::record::DeployRecord;
use crate::deploy::Deploy;
use crate::errors::CommonError;
use crate::mutated::pin::{PinError, RequestedPinned, RequestedPrefixDigest};
use crate::orchestrator::Context;
use crate::orchestrator::stage::{RollbackGuard, Stage};

pub struct SetPinnedStage;

impl Stage<PinError> for SetPinnedStage {
    fn run(
        &self, context: &mut Context, _cancel: &CancelToken, progress: ProgressEventBuilder,
    ) -> Result<(ProgressEventBuilder, Box<dyn RollbackGuard>), PinError> {
        let deploy = context.get::<Deploy>().ok_or(CommonError::MissingResult)?;
        let prefix_digest = context
            .get::<RequestedPrefixDigest>()
            .ok_or(CommonError::MissingResult)?;
        let pinned = context.get::<RequestedPinned>().ok_or(CommonError::MissingResult)?;

        let record_dir = deploy.deploy(&prefix_digest.0);
        let mut record = DeployRecord::read(&record_dir)?;

        let mut written = Vec::new();
        if replace(&mut record.pinned, pinned.0) != record.pinned {
            written.push(record.write(&record_dir)?);
        }

        Ok((progress, Box::new(written)))
    }
}
