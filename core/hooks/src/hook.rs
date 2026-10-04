// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::collections::HashMap;

use serde::Deserialize;

use super::error::HookError;
use super::primitive::{Primitive, Step};

#[cfg(test)]
#[path = "../tests/inline/hook.rs"]
mod tests;

#[derive(Debug, Clone, Deserialize)]
pub struct Hook {
    #[serde(default)]
    pub(crate) priority: i32,
    #[serde(default)]
    critical: bool,
    #[serde(default)]
    pub(crate) triggers: HashMap<String, Vec<String>>,
    #[serde(default)]
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

    pub fn execute(&mut self) -> Result<(), HookError> {
        while let Some(step) = self.steps.get_mut(self.executed) {
            match step.execute() {
                Ok(()) => self.executed += 1,
                Err(error) => {
                    if self.critical {
                        let _ = self.rollback();

                        return Err(error);
                    }

                    return Ok(());
                }
            }
        }

        Ok(())
    }

    pub fn rollback(&mut self) -> Result<(), HookError> {
        while let Some(index) = self.executed.checked_sub(1) {
            if let Some(step) = self.steps.get(index) {
                step.rollback()?;
            }

            self.executed = index;
        }

        Ok(())
    }
}
