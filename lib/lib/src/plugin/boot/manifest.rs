// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later

use std::collections::HashMap;
use std::fs;

use serde::Deserialize;

use crate::plugin::boot::error::BootPluginError;

#[derive(Debug, Clone, Deserialize)]
pub struct BootPluginManifest {
    pub name: String,
    pub library: String,
}

pub fn load_boot_plugin_manifests(
    boot_plugins_dir: &str, manifest_extension: &str,
) -> Result<HashMap<String, BootPluginManifest>, BootPluginError> {
    let mut manifests = HashMap::new();

    for entry in fs::read_dir(boot_plugins_dir)? {
        let path = entry?.path();

        if path.extension().and_then(|extension| extension.to_str()) != Some(manifest_extension) {
            continue;
        }

        let raw = fs::read_to_string(&path)?;
        let manifest: BootPluginManifest = toml::from_str(&raw)?;

        if manifests.contains_key(&manifest.name) {
            return Err(BootPluginError::DuplicateName(manifest.name));
        }

        manifests.insert(manifest.name.clone(), manifest);
    }

    Ok(manifests)
}
