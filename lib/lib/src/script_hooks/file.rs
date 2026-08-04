// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 JustPav
//
// SPDX-License-Identifier: LGPL-3.0-or-later

use std::collections::HashMap;
use std::path::PathBuf;

use serde::Deserialize;

use crate::script_hooks::error::HookError;
use crate::script_hooks::native::{NativeTrigger, Operation, Timing};

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Primitive {
    TouchFile { path: PathBuf },
    MoveFile { from: PathBuf, to: PathBuf },
    CreateSymlink { target: PathBuf, link: PathBuf },
}

#[derive(Debug, Clone, Deserialize)]
pub struct HookFile {
    #[serde(default)]
    pub priority: i32,
    #[serde(default)]
    pub critical: bool,
    pub operation: Option<Operation>,
    pub timing: Option<Timing>,
    #[serde(default)]
    pub triggers: HashMap<String, Vec<String>>,
    #[serde(default)]
    pub steps: Vec<Primitive>,
}

impl HookFile {
    pub fn parse(raw: &str) -> Result<Self, HookError> {
        let file: HookFile = toml::from_str(raw)?;

        match (file.operation, file.timing) {
            (Some(_), None) | (None, Some(_)) => return Err(HookError::InvalidTrigger),
            (None, None) if file.triggers.is_empty() => return Err(HookError::NoTrigger),
            _ => {}
        }

        Ok(file)
    }

    pub fn native_trigger(&self) -> Option<NativeTrigger> {
        match (self.operation, self.timing) {
            (Some(operation), Some(timing)) => Some(NativeTrigger { operation, timing }),
            _ => None,
        }
    }
}
