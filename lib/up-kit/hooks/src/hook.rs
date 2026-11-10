// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::collections::HashMap;
use std::os::fd::BorrowedFd;

use serde::Deserialize;

use super::error::HookError;
use super::primitive::{Primitive, Step};

#[cfg(test)]
#[path = "../tests/inline/hook.rs"]
mod tests;

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Hook {
    priority: i32,
    critical: bool,
    triggers: HashMap<String, Vec<String>>,
    steps: Vec<Primitive>,
    #[serde(skip)]
    executed: usize,
}

impl Hook {
    pub(crate) fn parse(raw: &str) -> Result<Self, HookError> {
        let hook: Hook = toml::from_str(raw)?;

        if hook.triggers.is_empty() {
            return Err(HookError::NoTrigger);
        }

        Ok(hook)
    }

    pub(crate) fn priority(&self) -> i32 {
        self.priority
    }

    pub(crate) fn triggers(&self) -> &HashMap<String, Vec<String>> {
        &self.triggers
    }

    pub(crate) fn execute(&mut self, root: BorrowedFd) -> Result<(), HookError> {
        while let Some(step) = self.steps.get_mut(self.executed) {
            match step.execute(root) {
                Ok(()) => self.executed += 1,
                Err(error) => {
                    if self.critical {
                        let _ = self.rollback(root);

                        return Err(error);
                    }

                    return Ok(());
                }
            }
        }

        Ok(())
    }

    pub(crate) fn rollback(&mut self, root: BorrowedFd) -> Result<(), HookError> {
        while let Some(index) = self.executed.checked_sub(1) {
            if let Some(step) = self.steps.get(index) {
                step.rollback(root)?;
            }

            self.executed = index;
        }

        Ok(())
    }
}
