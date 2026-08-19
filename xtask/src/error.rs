// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: GPL-3.0-only

use std::fmt;
use std::fmt::{Display, Formatter};
use std::io::Error as IoError;
use std::path::PathBuf;

use crate::gen_tree::splice::{MARKER_END, MARKER_START};

#[derive(Debug)]
pub enum XtaskError {
    Io(IoError),
    MissingCommand,
    UnknownCommand(String),
    UnknownArgument(String),
    MissingDepthValue,
    InvalidDepth(String),
    RepoRootNotFound,
    NoMarkedFiles(PathBuf),
    MissingStartMarker,
    MissingEndMarker,
    EndBeforeStart,
    DuplicateStartMarker,
    Splice { path: PathBuf, source: Box<XtaskError> },
}

impl From<IoError> for XtaskError {
    fn from(error: IoError) -> Self {
        XtaskError::Io(error)
    }
}

impl Display for XtaskError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            XtaskError::Io(error) => write!(f, "{error}"),
            XtaskError::MissingCommand => write!(f, "usage: cargo xtask gen-tree [--check] [--depth N]"),
            XtaskError::UnknownCommand(command) => {
                write!(
                    f,
                    "unknown command: {command}\nusage: cargo xtask gen-tree [--check] [--depth N]"
                )
            }
            XtaskError::UnknownArgument(argument) => write!(f, "unknown argument: {argument}"),
            XtaskError::MissingDepthValue => write!(f, "--depth needs an integer argument"),
            XtaskError::InvalidDepth(value) => write!(f, "--depth needs an integer argument, got {value:?}"),
            XtaskError::RepoRootNotFound => write!(f, "xtask must live directly under the repo root"),
            XtaskError::NoMarkedFiles(root) => write!(
                f,
                "no file under {} contains {MARKER_START} / {MARKER_END} — nothing to do",
                root.display()
            ),
            XtaskError::MissingStartMarker => write!(f, "missing {MARKER_START}"),
            XtaskError::MissingEndMarker => write!(f, "missing {MARKER_END}"),
            XtaskError::EndBeforeStart => write!(f, "{MARKER_END} appears before {MARKER_START}"),
            XtaskError::DuplicateStartMarker => write!(f, "more than one {MARKER_START} in file"),
            XtaskError::Splice { path, source } => write!(f, "{}: {source}", path.display()),
        }
    }
}
