// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later

use std::fs::read_to_string;
use std::path::Path;

use toml::from_str;

use upac_types::PackageMeta;

use crate::error::SetupError;
use crate::layout::meta::FILENAME;

pub fn read(source_dir: &Path, filename: Option<&str>) -> Result<PackageMeta, SetupError> {
    let content = read_to_string(source_dir.join(filename.unwrap_or(FILENAME)))?;

    Ok(from_str(&content)?)
}
