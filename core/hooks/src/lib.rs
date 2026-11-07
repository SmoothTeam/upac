// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::cmp::Reverse;
use std::collections::{BTreeMap, HashMap};
use std::os::fd::BorrowedFd;

use upac_types::CancelToken;
use upac_types::decoder::{PackageTriggers, TriggerPosition};

use self::error::HookError;
use self::hook::Hook;

mod layout {
    include!(concat!(env!("OUT_DIR"), "/layout.rs"));
}
mod hook;
mod primitive;

pub mod builder;
pub mod error;

#[derive(Debug)]
pub struct Hooks {
    hooks: BTreeMap<String, Hook>,
    triggers: HashMap<String, HashMap<String, String>>,
}

#[derive(Debug, Default)]
pub struct HookExecution {
    executed: Vec<Hook>,
}

impl HookExecution {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn execute(
        &mut self, hooks: &Hooks, position: TriggerPosition, packages: &[PackageTriggers], root: BorrowedFd,
        cancel: &CancelToken, progress: &dyn Fn(Option<&str>, u64, u64),
    ) -> Result<(), HookError> {
        let mut matched = BTreeMap::new();

        for package in packages {
            let Some(table) = hooks.triggers.get(&package.format) else {
                continue;
            };

            for trigger in package.triggers.iter().filter(|trigger| trigger.position == position) {
                let Some((hook_name, hook)) = table
                    .get(&trigger.name)
                    .and_then(|hook_name| hooks.hooks.get_key_value(hook_name))
                else {
                    continue;
                };

                matched.insert((Reverse(hook.priority()), hook_name.as_str()), hook);
            }
        }

        let total = matched.len() as u64;

        for (done, ((_, hook_name), hook)) in matched.into_iter().enumerate() {
            if cancel.is_cancelled() {
                return Err(HookError::Cancelled);
            }

            let mut running = hook.clone();
            let result = running.execute(root);
            self.executed.push(running);
            result?;

            progress(Some(hook_name), done as u64 + 1, total);
        }

        Ok(())
    }

    pub fn rollback(&mut self, root: BorrowedFd) -> Result<(), HookError> {
        while let Some(hook) = self.executed.last_mut() {
            hook.rollback(root)?;
            self.executed.pop();
        }

        Ok(())
    }
}
