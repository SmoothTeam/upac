// SPDX-FileCopyrightText: 2026 JustPav
//
// SPDX-License-Identifier: GPL-3.0-only

use std::fmt;
use std::fmt::{Display, Formatter};
use std::path::PathBuf;

pub struct Violation {
    pub path: PathBuf,
    pub line: usize,
    pub rule: &'static str,
    pub message: String,
}

impl Display for Violation {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}: [{}] {}", self.path.display(), self.line, self.rule, self.message)
    }
}
