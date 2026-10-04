// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::collections::{HashMap, HashSet};
use std::fs::{read, read_dir};
use std::path::Path;
use std::str::from_utf8;

use upac_types::decoder::{PackageTriggers, TriggerPosition};

use upac_pki::signature::{HookSignature, RootCertificate};

use self::error::HookError;
use self::hook::Hook;
use self::layout::hooks::{HOOK_EXTENSION, HOOKS_DIR, ROOT_CERT_PATH, SIGNATURE_EXTENSION};
use self::triggers::build_trigger_table;

mod layout {
    include!(concat!(env!("OUT_DIR"), "/layout.rs"));
}
mod primitive;
mod triggers;

pub mod error;
pub mod hook;

#[cfg(test)]
#[path = "../tests/inline/hooks.rs"]
mod tests;

pub struct Hooks {
    hooks: Vec<Hook>,
}

impl Hooks {
    pub fn load() -> Result<Self, HookError> {
        Self::load_from(Path::new(HOOKS_DIR), Path::new(ROOT_CERT_PATH))
    }

    pub fn load_from(hooks_dir: &Path, root_cert_path: &Path) -> Result<Self, HookError> {
        let root_bytes = read(root_cert_path)?;
        let root_certificate = RootCertificate::from_bytes(&root_bytes)?;

        let mut hooks = Vec::new();

        for entry in read_dir(hooks_dir)? {
            let path = entry?.path();

            if path.extension().and_then(|extension| extension.to_str()) != Some(HOOK_EXTENSION) {
                continue;
            }

            let mut signature_path = path.clone().into_os_string();
            signature_path.push(".");
            signature_path.push(SIGNATURE_EXTENSION);

            let hook_bytes = read(&path)?;
            let signature_bytes = read(&signature_path)?;

            let signature = HookSignature::from_bytes(&signature_bytes)?;
            signature.verify(&hook_bytes, &root_certificate)?;

            hooks.push(Hook::parse(from_utf8(&hook_bytes)?)?);
        }

        Ok(Self { hooks })
    }

    pub fn matching(self, position: TriggerPosition, packages: &[PackageTriggers]) -> Result<Vec<Hook>, HookError> {
        let mut tables = HashMap::new();
        for package in packages {
            if !tables.contains_key(&package.format) {
                tables.insert(
                    package.format.clone(),
                    build_trigger_table(&self.hooks, &package.format)?,
                );
            }
        }

        let mut matched_indices = HashSet::new();
        for package in packages {
            let Some(table) = tables.get(&package.format) else {
                continue;
            };

            for trigger in &package.triggers {
                if trigger.position != position {
                    continue;
                }

                if let Some(hook_index) = table.get(trigger.name.as_str()) {
                    matched_indices.insert(*hook_index);
                }
            }
        }

        Ok(self
            .hooks
            .into_iter()
            .enumerate()
            .filter(|(hook_index, _)| matched_indices.contains(hook_index))
            .map(|(_, hook)| hook)
            .collect())
    }
}
