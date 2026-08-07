// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 JustPav
//
// SPDX-License-Identifier: LGPL-3.0-or-later

use std::fs::{File, remove_file, rename};
use std::os::unix::fs::symlink;
use std::path::PathBuf;

use serde::Deserialize;
use upac_abi::error::ErrorKind;

use crate::orchestrator::stage::{RollbackGuard, StageResult};
use crate::scripts::error::HookError;

pub trait Step {
    fn execute(&mut self) -> Result<(), HookError>;
    fn rollback(&self) -> Result<(), HookError>;
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Primitive {
    TouchFile(TouchFile),
    MoveFile(MoveFile),
    CreateSymlink(CreateSymlink),
}

impl Step for Primitive {
    fn execute(&mut self) -> Result<(), HookError> {
        match self {
            Primitive::TouchFile(step) => step.execute(),
            Primitive::MoveFile(step) => step.execute(),
            Primitive::CreateSymlink(step) => step.execute(),
        }
    }

    fn rollback(&self) -> Result<(), HookError> {
        match self {
            Primitive::TouchFile(step) => step.rollback(),
            Primitive::MoveFile(step) => step.rollback(),
            Primitive::CreateSymlink(step) => step.rollback(),
        }
    }
}

impl RollbackGuard for Vec<Primitive> {
    fn new_none(_result: StageResult) -> Self {
        Vec::new()
    }

    fn rollback(&mut self) -> Result<(), ErrorKind> {
        while let Some(primitive) = self.pop() {
            primitive.rollback()?;
        }

        Ok(())
    }

    fn result(&self) -> StageResult {
        StageResult::Advance
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct TouchFile {
    pub path: PathBuf,

    #[serde(skip)]
    created: bool,
}

impl Step for TouchFile {
    fn execute(&mut self) -> Result<(), HookError> {
        if self.path.exists() {
            self.created = false;
            return Ok(());
        }

        File::create(&self.path)?;
        self.created = true;

        Ok(())
    }

    fn rollback(&self) -> Result<(), HookError> {
        if self.created {
            remove_file(&self.path)?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct MoveFile {
    pub from: PathBuf,
    pub to: PathBuf,
}

impl Step for MoveFile {
    fn execute(&mut self) -> Result<(), HookError> {
        Ok(rename(&self.from, &self.to)?)
    }

    fn rollback(&self) -> Result<(), HookError> {
        Ok(rename(&self.to, &self.from)?)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateSymlink {
    pub target: PathBuf,
    pub link: PathBuf,
}

impl Step for CreateSymlink {
    fn execute(&mut self) -> Result<(), HookError> {
        Ok(symlink(&self.target, &self.link)?)
    }

    fn rollback(&self) -> Result<(), HookError> {
        Ok(remove_file(&self.link)?)
    }
}
