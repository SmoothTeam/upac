// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::fs::read_to_string;
use std::path::Path;
use std::str::FromStr;

use mime::Mime;

use serde::Deserialize;

use super::error::DecoderError;

#[derive(Debug, Clone, Deserialize)]
pub struct DecoderManifest {
    pub format: String,
    pub extensions: Vec<String>,
    pub library: String,
    pub mime: String,
}

impl DecoderManifest {
    pub fn read(manifest_path: &Path) -> Result<Self, DecoderError> {
        let manifest: DecoderManifest = toml::from_str(&read_to_string(manifest_path)?)?;

        Mime::from_str(&manifest.mime)?;

        Ok(manifest)
    }
}
