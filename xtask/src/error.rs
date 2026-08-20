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
    ComponentsRequireStaticLink,
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
            XtaskError::ComponentsRequireStaticLink => {
                write!(f, "--components requires --link lib-static or --link full-static")
            }
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
