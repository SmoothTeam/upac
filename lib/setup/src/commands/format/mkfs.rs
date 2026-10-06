// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_types::CancelToken;
use upac_types::error::ErrorKind;

use upac_macro::stage;

use upac_orchestrator::context::Context;
use upac_orchestrator::stage::Stage;

use super::filesystem::FormatTarget;
use super::{RequestedDevice, RequestedFilesystem};

pub struct MkfsStage;

#[stage]
impl Stage<ErrorKind> for MkfsStage {
    fn run(
        &self, context: &mut Context, _cancel: &CancelToken, _progress: &dyn Fn(Option<&str>, u64, u64),
    ) -> Result<(), ErrorKind> {
        let filesystem = context.get::<RequestedFilesystem>()?;

        FormatTarget {
            device_path: &context.get::<RequestedDevice>()?.0,
            label: filesystem.label.as_deref(),
        }
        .format(
            filesystem.fs_kind,
            filesystem.btrfs_node_size,
            filesystem.btrfs_sector_size,
        )
    }
}
