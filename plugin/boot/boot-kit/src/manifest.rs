// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::fs::read_to_string;
use std::path::Path;

use serde::Deserialize;

use super::error::BooterError;

#[derive(Debug, Clone, Deserialize)]
pub struct BooterManifest {
    pub name: String,
    pub library: String,
}

impl BooterManifest {
    pub fn read(manifest_path: &Path) -> Result<Self, BooterError> {
        Ok(toml::from_str(&read_to_string(manifest_path)?)?)
    }
}
