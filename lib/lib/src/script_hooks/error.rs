// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 JustPav
//
// SPDX-License-Identifier: LGPL-3.0-or-later

use toml::de::Error as TomlError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HookError {
    Parse,
    InvalidTrigger,
    NoTrigger,
}

impl From<TomlError> for HookError {
    fn from(_: TomlError) -> Self {
        HookError::Parse
    }
}
