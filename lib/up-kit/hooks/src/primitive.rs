// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::ffi::OsStr;
use std::os::fd::{BorrowedFd, OwnedFd};
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::Command;

use nix::errno::Errno;
use nix::fcntl::{OFlag, OpenHow, ResolveFlag, openat, openat2, renameat};
use nix::sys::stat::Mode;
use nix::unistd::{UnlinkatFlags, chdir, chroot, fchdir, symlinkat, unlinkat};
use serde::Deserialize;

use super::error::HookError;
use super::layout::steps::{CHROOT_PATH, FILE_MODE, WORK_DIR};

pub(crate) trait Step {
    fn execute(&mut self, root: BorrowedFd) -> Result<(), HookError>;
    fn rollback(&self, root: BorrowedFd) -> Result<(), HookError>;
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub(crate) enum Primitive {
    TouchFile(TouchFile),
    MoveFile(MoveFile),
    CreateSymlink(CreateSymlink),
    Run(Run),
}

impl Step for Primitive {
    fn execute(&mut self, root: BorrowedFd) -> Result<(), HookError> {
        match self {
            Primitive::TouchFile(step) => step.execute(root),
            Primitive::MoveFile(step) => step.execute(root),
            Primitive::CreateSymlink(step) => step.execute(root),
            Primitive::Run(step) => step.execute(root),
        }
    }

    fn rollback(&self, root: BorrowedFd) -> Result<(), HookError> {
        match self {
            Primitive::TouchFile(step) => step.rollback(root),
            Primitive::MoveFile(step) => step.rollback(root),
            Primitive::CreateSymlink(step) => step.rollback(root),
            Primitive::Run(step) => step.rollback(root),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct TouchFile {
    path: PathBuf,

    #[serde(skip)]
    created: bool,
}

impl Step for TouchFile {
    fn execute(&mut self, root: BorrowedFd) -> Result<(), HookError> {
        let (parent, name) = parent_open(root, &self.path)?;

        match openat(
            &parent,
            name,
            OFlag::O_CREAT | OFlag::O_EXCL | OFlag::O_WRONLY | OFlag::O_CLOEXEC,
            Mode::from_bits_truncate(FILE_MODE),
        ) {
            Ok(_) => self.created = true,
            Err(Errno::EEXIST) => self.created = false,
            Err(error) => return Err(error.into()),
        }

        Ok(())
    }

    fn rollback(&self, root: BorrowedFd) -> Result<(), HookError> {
        if self.created {
            let (parent, name) = parent_open(root, &self.path)?;
            unlinkat(&parent, name, UnlinkatFlags::NoRemoveDir)?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct MoveFile {
    from: PathBuf,
    to: PathBuf,
}

impl Step for MoveFile {
    fn execute(&mut self, root: BorrowedFd) -> Result<(), HookError> {
        let (from_parent, from_name) = parent_open(root, &self.from)?;
        let (to_parent, to_name) = parent_open(root, &self.to)?;

        Ok(renameat(&from_parent, from_name, &to_parent, to_name)?)
    }

    fn rollback(&self, root: BorrowedFd) -> Result<(), HookError> {
        let (from_parent, from_name) = parent_open(root, &self.from)?;
        let (to_parent, to_name) = parent_open(root, &self.to)?;

        Ok(renameat(&to_parent, to_name, &from_parent, from_name)?)
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct CreateSymlink {
    target: PathBuf,
    link: PathBuf,
}

impl Step for CreateSymlink {
    fn execute(&mut self, root: BorrowedFd) -> Result<(), HookError> {
        let (parent, name) = parent_open(root, &self.link)?;

        Ok(symlinkat(&self.target, &parent, name)?)
    }

    fn rollback(&self, root: BorrowedFd) -> Result<(), HookError> {
        let (parent, name) = parent_open(root, &self.link)?;

        Ok(unlinkat(&parent, name, UnlinkatFlags::NoRemoveDir)?)
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Run {
    command: String,
    args: Vec<String>,
}

impl Step for Run {
    fn execute(&mut self, root: BorrowedFd) -> Result<(), HookError> {
        let command_root = root.try_clone_to_owned()?;

        let mut command = Command::new(&self.command);
        command.args(&self.args);

        unsafe {
            command.pre_exec(move || {
                fchdir(&command_root)?;
                chroot(CHROOT_PATH)?;
                chdir(WORK_DIR)?;

                Ok(())
            });
        }

        let status = command.status()?;

        if !status.success() {
            return Err(HookError::Command(status.code()));
        }

        Ok(())
    }

    fn rollback(&self, _root: BorrowedFd) -> Result<(), HookError> {
        Ok(())
    }
}

fn parent_open<'path>(root: BorrowedFd, path: &'path Path) -> Result<(OwnedFd, &'path OsStr), HookError> {
    if !path.is_absolute() {
        return Err(HookError::InvalidPath);
    }

    let name = path.file_name().ok_or(HookError::InvalidPath)?;
    let parent = path.parent().ok_or(HookError::InvalidPath)?;

    let how = OpenHow::new()
        .flags(OFlag::O_PATH | OFlag::O_DIRECTORY | OFlag::O_CLOEXEC)
        .resolve(ResolveFlag::RESOLVE_IN_ROOT | ResolveFlag::RESOLVE_NO_MAGICLINKS);

    Ok((openat2(root, parent, how)?, name))
}
