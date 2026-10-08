// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::fs::{File, read_dir};
use std::path::Path;

use composefs::generic_tree::Stat;

use upac_types::CancelToken;
use upac_types::error::ErrorKind;
use upac_types::response::bootstrap::SetupBootstrapKernelResponse;

use upac_composefs::tree::Tree;

use upac_macro::stage;

use upac_orchestrator::context::Context;
use upac_orchestrator::stage::Stage;

use super::{KernelImage, KernelVersion};

use crate::layout::bootstrap::{MODULE_INDEX_PREFIX, MODULES_DIR};

pub struct CommitStage;

#[stage]
impl Stage<ErrorKind> for CommitStage {
    fn run(
        &self, context: &mut Context, _cancel: &CancelToken, _progress: &dyn Fn(Option<&str>, u64, u64),
    ) -> Result<(), ErrorKind> {
        let mut prefix_tree = context.take::<Tree>()?;
        let image_path = context.take::<KernelImage>()?.0;
        let modules_dir = Path::new(MODULES_DIR).join(&context.get::<KernelVersion>()?.0);

        let exported_modules_dir = image_path.parent().ok_or(ErrorKind::InvalidPath)?;
        for entry in read_dir(exported_modules_dir)? {
            let entry = entry?;
            let is_module_index = entry
                .file_name()
                .to_str()
                .is_some_and(|name| name.starts_with(MODULE_INDEX_PREFIX));

            if is_module_index && entry.file_type()?.is_file() {
                prefix_tree.insert_file(
                    modules_dir.join(entry.file_name()),
                    &File::open(entry.path())?,
                    Stat::uninitialized(),
                )?;
            }
        }

        let image_name = image_path.file_name().ok_or(ErrorKind::InvalidPath)?;
        prefix_tree.insert_file(
            modules_dir.join(image_name),
            &File::open(&image_path)?,
            Stat::uninitialized(),
        )?;

        context.put(SetupBootstrapKernelResponse {
            prefix_digest: prefix_tree.commit()?.to_hex(),
        });

        Ok(())
    }
}
