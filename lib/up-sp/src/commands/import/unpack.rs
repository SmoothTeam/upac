// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_types::CancelToken;
use upac_types::error::ErrorKind;

use upac_decoder_loader::unpack::PackageUnpacker;

use upac_macro::stage;

use upac_orchestrator::context::Context;
use upac_orchestrator::stage::Stage;

use super::{PackageSource, ScratchDir, UnpackedPackage};

pub struct UnpackStage;

#[stage]
impl Stage<ErrorKind> for UnpackStage {
    fn run(
        &self, context: &mut Context, cancel: &CancelToken, progress: &dyn Fn(Option<&str>, u64, u64),
    ) -> Result<(), ErrorKind> {
        let source = context.take::<PackageSource>()?;
        let scratch_dir = context.get::<ScratchDir>()?.0.to_string_lossy().into_owned();
        let mut unpacker = context.take::<PackageUnpacker>()?;

        let source_path = source.path.to_string_lossy();
        let package_file_name = source
            .path
            .file_name()
            .unwrap_or(source.path.as_os_str())
            .to_string_lossy();
        progress(Some(&package_file_name), 0, 0);
        let unpacked = unpacker.unpack_one(&source_path, source.index, &scratch_dir, cancel);
        context.put(unpacker);

        let (temp, triggers) = unpacked?;
        context.put(UnpackedPackage { temp, triggers });

        Ok(())
    }
}
