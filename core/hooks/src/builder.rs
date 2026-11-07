// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::cmp::Reverse;
use std::collections::{BTreeMap, HashMap};
use std::fs::{read, read_dir};
use std::path::Path;
use std::str::from_utf8;

use upac_pki::signature::{HookSignature, RootCertificate};

use super::Hooks;
use super::error::HookError;
use super::hook::Hook;
use super::layout::hooks::{HOOK_EXTENSION, SIGNATURE_EXTENSION};

#[cfg(test)]
#[path = "../tests/inline/builder.rs"]
mod tests;

#[derive(Debug, Default)]
pub struct HooksBuilder {
    hooks: BTreeMap<String, Hook>,
}

impl HooksBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn load(&mut self, hooks_dir: &Path, root_cert_path: &Path) -> Result<(), HookError> {
        let root_bytes = read(root_cert_path)?;
        let root_certificate = RootCertificate::from_bytes(&root_bytes)?;

        let mut loaded = BTreeMap::new();

        for entry in read_dir(hooks_dir)? {
            let path = entry?.path();

            if path.extension().and_then(|extension| extension.to_str()) != Some(HOOK_EXTENSION) {
                continue;
            }

            let name = path
                .file_stem()
                .and_then(|stem| stem.to_str())
                .ok_or(HookError::Encoding)?
                .to_owned();

            if self.hooks.contains_key(&name) {
                return Err(HookError::DuplicateHook(name));
            }

            let mut signature_path = path.clone().into_os_string();
            signature_path.push(".");
            signature_path.push(SIGNATURE_EXTENSION);

            let hook_bytes = read(&path)?;
            let signature_bytes = read(&signature_path)?;

            let signature = HookSignature::from_bytes(&signature_bytes)?;
            signature.verify(&hook_bytes, &root_certificate)?;

            loaded.insert(name, Hook::parse(from_utf8(&hook_bytes)?)?);
        }

        self.hooks.extend(loaded);

        Ok(())
    }

    pub fn build(self) -> Result<Hooks, HookError> {
        let mut by_priority: Vec<(&String, &Hook)> = self.hooks.iter().collect();
        by_priority.sort_by_key(|(_, hook)| Reverse(hook.priority()));

        let mut triggers: HashMap<String, HashMap<String, String>> = HashMap::new();

        for (hook_name, hook) in by_priority {
            for (format, trigger_names) in hook.triggers() {
                let table = triggers.entry(format.clone()).or_default();

                for trigger_name in trigger_names {
                    let Some(winner_name) = table.get(trigger_name) else {
                        table.insert(trigger_name.clone(), hook_name.clone());
                        continue;
                    };

                    if self.hooks.get(winner_name).map(Hook::priority) == Some(hook.priority()) {
                        return Err(HookError::TriggerConflict {
                            format: format.clone(),
                            name: trigger_name.clone(),
                            hooks: (winner_name.clone(), hook_name.clone()),
                        });
                    }
                }
            }
        }

        Ok(Hooks {
            hooks: self.hooks,
            triggers,
        })
    }
}
