use anyhow::Result;

use std::env::temp_dir;
use std::ffi::CString;
use std::sync::Arc;

use crate::config::Config;
use crate::corelib::Lib;

use crate::ffi::DiffKind;

pub mod backend;
pub mod commit;
pub mod errors;
pub mod package;

pub const EXPECTED_ABI_VERSION: u32 = 2;

pub struct PackageDiffEntry {
    pub name: String,
    pub kind: DiffKind,
}

pub struct FileDiffEntry {
    pub path: String,
    pub kind: DiffKind,
    pub package_name: String,
}

pub struct CommandContext {
    pub config: Config,
    pub lib: Arc<Lib>,
    pub tmp_path: CString,
}

impl CommandContext {
    pub fn new(config: Config, lib: Arc<Lib>) -> Result<CommandContext> {
        let tmp_path = CString::new(temp_dir().to_string_lossy().as_ref())?;

        Ok(CommandContext {
            config,
            lib,
            tmp_path,
        })
    }
}
