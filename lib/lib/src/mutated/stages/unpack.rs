// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::cell::RefCell;
use std::fs::remove_dir_all;
use std::io::ErrorKind as IoErrorKind;
use std::path::PathBuf;

use upac_types::CancelToken;
use upac_types::decoder::PackageTriggers;
use upac_types::error::ErrorKind;

use upac_decoder_loader::unpack::PackageUnpacker;

use upac_macro::stage;

use upac_orchestrator::context::Context;
use upac_orchestrator::stage::Stage;

use super::super::TmpPath;
use super::{PackageSource, UnpackedPackage};

#[derive(Default)]
pub struct UnpackStage {
    unpacked_dirs: RefCell<Vec<PathBuf>>,
}

#[stage]
impl Stage<ErrorKind> for UnpackStage {
    fn run(
        &self, context: &mut Context, cancel: &CancelToken, _progress: &dyn Fn(Option<&str>, u64, u64),
    ) -> Result<(), ErrorKind> {
        let source = context.take::<PackageSource>()?;
        let tmp_path = context.get::<TmpPath>()?.0.clone();
        let mut unpacker = context.take::<PackageUnpacker>()?;

        let unpacked = unpacker.unpack_one(&source.path, source.index, &tmp_path, cancel);
        context.put(unpacker);
        let (temp, triggers) = unpacked?;
        self.unpacked_dirs
            .borrow_mut()
            .push(PathBuf::from(&temp.temp_package_path));

        context.push::<PackageTriggers>(triggers.clone());
        context.push(UnpackedPackage { temp, triggers });

        Ok(())
    }

    fn rollback(&self, _context: &mut Context) -> Result<(), ErrorKind> {
        for unpacked_dir in self.unpacked_dirs.borrow_mut().drain(..) {
            match remove_dir_all(&unpacked_dir) {
                Ok(()) => {}
                Err(error) if error.kind() == IoErrorKind::NotFound => {}
                Err(error) => return Err(error.into()),
            }
        }

        Ok(())
    }
}
